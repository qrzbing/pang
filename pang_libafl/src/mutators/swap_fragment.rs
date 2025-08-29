use std::{borrow::Cow, collections::HashMap, sync::Arc};

use libafl::{
    HasMetadata,
    corpus::CorpusId,
    mutators::{MutationResult, Mutator},
    state::HasRand,
};
use libafl_bolts::{Error, Named, rands::Rand};

use log::debug;
use pang::{language::Language, symbol::Symbol, tree::DerivationTree};

use crate::{input::PangInput, mutators::PangHelper, state::PangMutateState};

/// # SwapFragmentMutator
///
/// Select a leaf node and replace it with a random fragment from the pool.
#[derive(Debug)]
pub struct SwapFragmentMutator {
    lang: Arc<Language>,
}

impl SwapFragmentMutator {
    /// Creates a new [`SwapFragmentMutator`]
    #[must_use]
    pub fn new(lang: &Arc<Language>) -> Self {
        Self { lang: lang.clone() }
    }

    fn swap_region<S>(&self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error>
    where
        S: HasRand + HasMetadata,
    {
        let fragments = state
            .metadata::<PangMutateState>()
            .unwrap()
            .fragments
            .clone();
        let regions = input.regions.as_ref().unwrap();

        let available_keys: Vec<_> = regions
            .keys()
            .filter(|&key| {
                // `len(seed.regions[r]) > 0`
                let has_regions = regions.get(key).map_or(false, |r| !r.is_empty());
                // `len(self.fragments[r]) > 0`
                // Now use the cloned `fragments`
                let has_fragments = fragments.get(key).map_or(false, |f| !f.is_empty());
                has_regions && has_fragments
            })
            .collect();

        if available_keys.is_empty() {
            debug!("No regions with available fragments");
            return Ok(MutationResult::Skipped);
        }
        let rng = state.rand_mut();

        let key = rng.choose(&available_keys).unwrap();

        let region_set = regions.get(*key).unwrap().iter().collect::<Vec<_>>();
        let region_to_swap = rng.choose(&region_set).unwrap();
        let (start, end) = (region_to_swap.start, region_to_swap.end);

        let fragment_pool = fragments.get(*key).unwrap();
        let fragment_to_swap = rng.choose(fragment_pool).unwrap();
        let swap_bytes = fragment_to_swap.to_bytes();

        let mut new_bytes = Vec::with_capacity(start + swap_bytes.len() + input.bytes.len() - end);
        new_bytes.extend_from_slice(&input.bytes[..start]);
        new_bytes.extend_from_slice(&swap_bytes);
        new_bytes.extend_from_slice(&input.bytes[end..]);

        if input.bytes == new_bytes {
            debug!("No change in region");
            return Ok(MutationResult::Skipped);
        }

        input.bytes = new_bytes;
        // Reset structure and region info, as they are now invalid.
        // `new_seed.has_structure = False; new_seed.has_regions = False`
        input.has_structure = false;
        input.structure = None;
        input.has_regions = false;
        input.regions = None;

        Ok(MutationResult::Mutated)
    }
}

impl Named for SwapFragmentMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("SwapFragmentMutator")
    }
}

/// Try to find a random replacement fragment for the given tree node
/// Returns Some(fragment) if a suitable fragment exists, None otherwise
fn try_swap_fragment<R: Rand>(
    tree: &Arc<DerivationTree>,
    fragments: &HashMap<String, Vec<Arc<DerivationTree>>>,
    rng: &mut R,
) -> Option<Arc<DerivationTree>> {
    match &tree.symbol {
        Symbol::NonTerminal { label } => fragments
            .get(label)
            .filter(|frags| !frags.is_empty())
            .map(|frags| {
                let idx = rng.below_or_zero(frags.len());
                frags[idx].clone()
            }),
        _ => None,
    }
}

/// Swap a node at the specified target position with a random fragment
/// Returns a new tree with the node replaced, or original tree if no swap occurred
fn swap_node<R>(
    lang: &Language,
    tree: Arc<DerivationTree>,
    fragments: &HashMap<String, Vec<Arc<DerivationTree>>>,
    target: usize,
    rng: &mut R,
) -> Arc<DerivationTree>
where
    R: Rand,
{
    // Context to track current position and whether swap has occurred
    struct SwapContext {
        position: usize,
        swapped: bool,
    }

    // Recursive helper function to traverse tree and perform swap at target position
    fn do_swap<R>(
        lang: &Language,
        tree: Arc<DerivationTree>,
        fragments: &HashMap<String, Vec<Arc<DerivationTree>>>,
        target: usize,
        rng: &mut R,
        ctx: &mut SwapContext,
    ) -> Arc<DerivationTree>
    where
        R: Rand,
    {
        // Skip excluded nodes
        if lang.is_excluded(&tree.symbol) {
            return tree;
        }

        ctx.position += 1;

        // Perform swap if we've reached the target position
        if ctx.position == target {
            ctx.swapped = true;
            return try_swap_fragment(&tree, fragments, rng).unwrap_or(tree);
        }

        // Recursively process children if swap hasn't occurred yet
        tree.children
            .as_ref()
            .filter(|_| !ctx.swapped)
            .and_then(|children| {
                let new_children: Vec<_> = children
                    .iter()
                    .map(|child| do_swap(lang, child.clone(), fragments, target, rng, ctx))
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

    let mut ctx = SwapContext {
        position: 0,
        swapped: false,
    };
    do_swap(lang, tree, fragments, target, rng, &mut ctx)
}

impl<S> Mutator<PangInput, S> for SwapFragmentMutator
where
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error> {
        // Add the input to the fragment pool if it's not seen before.
        let pang_state = state
            .metadata_mut::<PangMutateState>()
            .expect("PangMutateState not found");

        if pang_state.add_seen_seed(input) {
            PangHelper::add_to_fragment_pool(&self.lang, pang_state, input);
        }

        if !input.has_structure && input.has_regions {
            return self.swap_region(state, input);
        }

        if !input.has_structure {
            return Ok(MutationResult::Skipped);
        }

        let tree = match &input.structure {
            Some(tree) => tree,
            None => return Ok(MutationResult::Skipped),
        };

        let n_nodes = PangHelper::count_nodes(&self.lang, tree);

        if n_nodes <= 1 {
            return Ok(MutationResult::Skipped);
        }

        let fragments = state
            .metadata::<PangMutateState>()
            .expect("PangMutateState not found")
            .fragments
            .clone();

        let rng = state.rand_mut();

        let to_swap = rng.between(2, n_nodes);

        let new_tree = swap_node(
            &self.lang,
            tree.clone(),
            &fragments,
            to_swap,
            state.rand_mut(),
        );

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
