use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use libafl::{
    HasMetadata,
    corpus::CorpusId,
    mutators::{MutationResult, Mutator},
    state::HasRand,
};
use libafl_bolts::{Error, Named, rands::Rand};

use log::debug;
use pang::{parser::Parser, tree::DerivationTree};

use crate::{input::PangInput, mutators::PangHelper, state::PangMutateState};

/// # DeleteFragmentMutator
///
/// Select a random node and delete its children (make it a leaf node).
#[derive(Debug)]
pub struct DeleteFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    parser: Arc<Mutex<P>>,
}

impl<P> DeleteFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    /// Creates a new [`DeleteFragmentMutator`].
    #[must_use]
    pub fn new(parser: Arc<Mutex<P>>) -> Self {
        Self { parser }
    }

    fn delete_region<S>(
        &self,
        state: &mut S,
        input: &mut PangInput,
    ) -> Result<MutationResult, Error>
    where
        S: HasRand + HasMetadata,
    {
        let regions = input.regions.as_ref().unwrap();

        // Find all symbols that have at least one region associated with them.
        let available_keys: Vec<_> = regions
            .keys()
            .filter(|&key| !regions.get(key).unwrap().is_empty())
            .cloned()
            .collect();

        if available_keys.is_empty() {
            debug!("No regions with available fragments");
            return Ok(MutationResult::Skipped);
        }
        let rng = state.rand_mut();

        let key = rng.choose(&available_keys).unwrap();
        let valid_regions_to_delete: Vec<_> = regions
            .get(key)
            .unwrap()
            .iter()
            .filter(|region| region.end - region.start >= 2)
            .collect();
        if valid_regions_to_delete.is_empty() {
            debug!("No regions with length >= 2 found for key '{}'", key);
            return Ok(MutationResult::Skipped);
        }
        let region_to_delete = rng.choose(&valid_regions_to_delete).unwrap();
        let (start, end) = (region_to_delete.start, region_to_delete.end);

        let mut new_bytes = Vec::with_capacity(input.bytes.len() - (end - start));
        new_bytes.extend_from_slice(&input.bytes[..start]);
        new_bytes.extend_from_slice(&input.bytes[end..]);

        // It's possible (though unlikely) that deleting a region results in no change.
        if input.bytes == new_bytes {
            return Ok(MutationResult::Skipped);
        }

        input.bytes = new_bytes;

        // Reset structure and region info, as they are now invalid.
        input.has_structure = false;
        input.structure = None;
        input.has_regions = false;
        input.regions = None;

        Ok(MutationResult::Mutated)
    }
}

impl<P> Named for DeleteFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("DeleteFragmentMutator")
    }
}

/// Delete children of a node at the specified target position
/// Returns a new tree with the node's children deleted, or original tree if no deletion occurred
fn delete_node<P>(parser: &P, tree: Arc<DerivationTree>, target: usize) -> Arc<DerivationTree>
where
    P: Parser,
{
    // Context to track current position and whether deletion has occurred
    struct DeleteContext {
        position: usize,
        deleted: bool,
    }

    // Recursive helper function to traverse tree and perform deletion at target position
    fn do_delete<P>(
        parser: &P,
        tree: Arc<DerivationTree>,
        target: usize,
        ctx: &mut DeleteContext,
    ) -> Arc<DerivationTree>
    where
        P: Parser,
    {
        // Skip excluded nodes
        if parser.is_excluded(&tree.symbol) {
            return tree;
        }

        ctx.position += 1;

        // Perform deletion if we've reached the target position
        if ctx.position == target {
            ctx.deleted = true;
            // Create a new node with empty children
            return Arc::new(DerivationTree {
                symbol: tree.symbol.clone(),
                children: Some(Vec::new()),
            });
        }

        // Recursively process children if deletion hasn't occurred yet
        tree.children
            .as_ref()
            .filter(|_| !ctx.deleted)
            .and_then(|children| {
                let new_children: Vec<_> = children
                    .iter()
                    .map(|child| do_delete(parser, child.clone(), target, ctx))
                    .collect();

                // Create new tree only if any child was modified
                if children
                    .iter()
                    .zip(&new_children)
                    .any(|(old, new)| !Arc::ptr_eq(old, new))
                {
                    Some(Arc::new(DerivationTree {
                        symbol: tree.symbol.clone(),
                        children: Some(new_children),
                    }))
                } else {
                    None
                }
            })
            .unwrap_or(tree)
    }

    let mut ctx = DeleteContext {
        position: 0,
        deleted: false,
    };
    do_delete(parser, tree, target, &mut ctx)
}

impl<P, S> Mutator<PangInput, S> for DeleteFragmentMutator<P>
where
    P: Parser + Send + Sync,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error> {
        // Add the input to the fragment pool if it's not seen before.
        let pang_state = state
            .metadata_mut::<PangMutateState>()
            .expect("PangMutateState not found");

        if pang_state.add_seen_seed(input) {
            PangHelper::add_to_fragment_pool(self.parser.clone(), pang_state, input);
        }

        if !input.has_structure && input.has_regions {
            return self.delete_region(state, input);
        }

        if !input.has_structure {
            return Ok(MutationResult::Skipped);
        }

        let parser = self.parser.lock().unwrap();

        let tree = match &input.structure {
            Some(tree) => tree,
            None => return Ok(MutationResult::Skipped),
        };

        let n_nodes = PangHelper::count_nodes(&*parser, tree);

        if n_nodes <= 1 {
            return Ok(MutationResult::Skipped);
        }

        let rng = state.rand_mut();
        let to_delete = rng.between(2, n_nodes);
        debug!("Deleting node at position {}", to_delete);

        let new_tree = delete_node(&*parser, tree.clone(), to_delete);

        let new_bytes = new_tree.to_bytes();

        // Check if the mutation resulted in an empty input
        if new_bytes.is_empty() {
            return Ok(MutationResult::Skipped);
        }

        if input.bytes == new_bytes {
            return Ok(MutationResult::Skipped);
        }

        input.bytes = new_bytes;
        input.structure = Some(new_tree);

        Ok(MutationResult::Mutated)
    }

    // For now, post_exec does nothing.
    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
