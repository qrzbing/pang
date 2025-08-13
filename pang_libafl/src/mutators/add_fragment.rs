use std::{
    borrow::Cow,
    collections::HashMap,
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
use pang::{grammar::Symbol, parser::Parser, tree::DerivationTree};

use crate::{input::PangInput, mutators::PangHelper, state::PangMutateState};

/// # AddFragmentMutator
///
/// Select a non-terminal node and add a compatible fragment as a new child.
#[derive(Debug)]
pub struct AddFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    parser: Arc<Mutex<P>>,
}

impl<P> AddFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    /// Creates a new [`AddFragmentMutator`]
    #[must_use]
    pub fn new(parser: Arc<Mutex<P>>) -> Self {
        Self { parser }
    }
}

impl<P> Named for AddFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("AddFragmentMutator")
    }
}

/// Count the number of non-terminal nodes that can accept new children
fn count_addable_nodes<P>(parser: &P, tree: &DerivationTree) -> usize
where
    P: Parser,
{
    if parser.is_excluded(&tree.symbol) {
        return 0;
    }

    let mut count = 0;

    // Only non-terminals can have children added
    if let Symbol::NonTerminal { .. } = &tree.symbol {
        count = 1;
    }

    if let Some(children) = &tree.children {
        for child in children {
            count += count_addable_nodes(parser, child);
        }
    }

    count
}

/// Add a fragment to a node at the specified target position
/// Returns a new tree with the fragment added, or original tree if no addition occurred
fn add_node<P, R>(
    parser: &P,
    tree: Arc<DerivationTree>,
    fragments: &HashMap<String, Vec<Arc<DerivationTree>>>,
    target: usize,
    rng: &mut R,
) -> Arc<DerivationTree>
where
    P: Parser,
    R: Rand,
{
    // Context to track current position and whether addition has occurred
    struct AddContext {
        position: usize,
        added: bool,
    }

    // Recursive helper function to traverse tree and perform addition at target position
    fn do_add<P, R>(
        parser: &P,
        tree: Arc<DerivationTree>,
        fragments: &HashMap<String, Vec<Arc<DerivationTree>>>,
        target: usize,
        rng: &mut R,
        ctx: &mut AddContext,
    ) -> Arc<DerivationTree>
    where
        P: Parser,
        R: Rand,
    {
        // Skip excluded nodes
        if parser.is_excluded(&tree.symbol) {
            return tree;
        }

        // Only count and potentially add to non-terminals
        if let Symbol::NonTerminal { label } = &tree.symbol {
            ctx.position += 1;

            // Perform addition if we've reached the target position
            if ctx.position == target {
                ctx.added = true;

                // Try to find a compatible fragment
                if let Some(frags) = fragments.get(label) {
                    if !frags.is_empty() {
                        let idx = rng.below_or_zero(frags.len());
                        let new_fragment = frags[idx].clone();

                        // Add the fragment to existing children
                        let mut new_children = tree.children.clone().unwrap_or_default();
                        new_children.push(new_fragment);

                        return Arc::new(DerivationTree {
                            symbol: tree.symbol.clone(),
                            children: Some(new_children),
                            value: None,
                        });
                    }
                }
                // If no suitable fragment found, return original
                return tree;
            }
        }

        // Recursively process children if addition hasn't occurred yet
        tree.children
            .as_ref()
            .filter(|_| !ctx.added)
            .and_then(|children| {
                let new_children: Vec<_> = children
                    .iter()
                    .map(|child| do_add(parser, child.clone(), fragments, target, rng, ctx))
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
                        value: None,
                    }))
                } else {
                    None
                }
            })
            .unwrap_or(tree)
    }

    let mut ctx = AddContext {
        position: 0,
        added: false,
    };
    do_add(parser, tree, fragments, target, rng, &mut ctx)
}

fn add_region_fragment<S>(state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error>
where
    S: HasRand + HasMetadata,
{
    // Clone fragments from state to release the borrow on `state`.
    let fragments = state
        .metadata::<PangMutateState>()
        .unwrap()
        .fragments
        .clone();
    let regions = input.regions.as_ref().unwrap();

    // Find all symbols that have both a region in the input and a fragment in the pool.
    // This logic is identical to the one in region-based swap.
    let available_keys: Vec<_> = regions
        .keys()
        .filter(|&key| {
            let has_regions = regions.get(key).map_or(false, |r| !r.is_empty());
            let has_fragments = fragments.get(key).map_or(false, |f| !f.is_empty());
            has_regions && has_fragments
        })
        .cloned()
        .collect();

    if available_keys.is_empty() {
        debug!("No regions with available fragments for adding.");
        return Ok(MutationResult::Skipped);
    }

    let rng = state.rand_mut();

    // Randomly choose a symbol to work with.
    let key = rng.choose(&available_keys).unwrap();

    // Randomly choose a region for that symbol. This will be our insertion point.
    let region_set = regions.get(key).unwrap().iter().collect::<Vec<_>>();
    let region_to_add_at = rng.choose(&region_set).unwrap();
    let insertion_point = region_to_add_at.start;

    // Randomly choose a compatible fragment to insert.
    let fragment_pool = fragments.get(key).unwrap();
    let fragment_to_add = rng.choose(fragment_pool).unwrap();
    let bytes_to_add = fragment_to_add.to_bytes();

    if bytes_to_add.is_empty() {
        return Ok(MutationResult::Skipped);
    }

    // Perform the insertion.
    let mut new_bytes = Vec::with_capacity(input.bytes.len() + bytes_to_add.len());
    new_bytes.extend_from_slice(&input.bytes[..insertion_point]);
    new_bytes.extend_from_slice(&bytes_to_add);
    new_bytes.extend_from_slice(&input.bytes[insertion_point..]);

    input.bytes = new_bytes;

    // Reset structure and region info, as they are now invalid.
    input.has_structure = false;
    input.structure = None;
    input.has_regions = false;
    input.regions = None;

    Ok(MutationResult::Mutated)
}

impl<P, S> Mutator<PangInput, S> for AddFragmentMutator<P>
where
    P: Parser + Send + Sync,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error> {
        let pang_state = state
            .metadata_mut::<PangMutateState>()
            .expect("PangMutateState not found");

        if pang_state.add_seen_seed(input) {
            PangHelper::add_to_fragment_pool(self.parser.clone(), pang_state, input);
        }

        if !input.has_structure && input.has_regions {
            return add_region_fragment(state, input);
        }

        if !input.has_structure {
            return Ok(MutationResult::Skipped);
        }

        let tree = match &input.structure {
            Some(tree) => tree,
            None => return Ok(MutationResult::Skipped),
        };

        let parser = self.parser.lock().unwrap();

        let n_addable = count_addable_nodes(&*parser, tree);

        if n_addable == 0 {
            return Ok(MutationResult::Skipped);
        }

        let fragments = state
            .metadata::<PangMutateState>()
            .expect("PangMutateState not found")
            .fragments
            .clone();

        let rng = state.rand_mut();

        let to_add = rng.between(1, n_addable);
        debug!("Adding fragment at position {}", to_add);

        let new_tree = add_node(&*parser, tree.clone(), &fragments, to_add, state.rand_mut());

        let new_bytes = new_tree.to_bytes();
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
