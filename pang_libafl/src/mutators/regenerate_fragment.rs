use std::{borrow::Cow, sync::Arc};

use libafl::{
    HasMetadata,
    corpus::CorpusId,
    mutators::{MutationResult, Mutator},
    state::HasRand,
};
use libafl_bolts::{Error, Named, rands::Rand};

use pang::{language::Language, symbol::Symbol, tree::DerivationTree};

use crate::{input::PangInput, mutators::PangHelper, state::PangMutateState};

/// # RegenerateFragmentMutator
///
/// Select a random node and regenerate its subtree.
#[derive(Debug)]
pub struct RegenerateFragmentMutator {
    lang: Arc<Language>,
}

impl RegenerateFragmentMutator {
    /// Creates a new [`RegenerateFragmentMutator`].
    #[must_use]
    pub fn new(lang: &Arc<Language>) -> Self {
        Self { lang: lang.clone() }
    }
}

impl Named for RegenerateFragmentMutator {
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("RegenerateFragmentMutator")
    }
}

fn regenerate_node_at<R: Rand>(
    lang: &Arc<Language>,
    tree: Arc<DerivationTree>,
    target: usize,
    rng: &mut R,
) -> Arc<DerivationTree> {
    struct RegenerateContext {
        position: usize,
        regenerated: bool,
    }

    fn do_regenerate<R: Rand>(
        lang: &Arc<Language>,
        tree: Arc<DerivationTree>,
        target: usize,
        ctx: &mut RegenerateContext,
        rng: &mut R,
    ) -> Arc<DerivationTree> {
        // Skip excluded nodes
        if lang.is_excluded(&tree.symbol) {
            return tree;
        }

        ctx.position += 1;

        if ctx.position == target {
            ctx.regenerated = true;
            // For non-terminals, generate a new subtree
            match &tree.symbol {
                Symbol::NonTerminal { label } => {
                    return lang.grammar.generate_combinator(label, rng, &[]);
                }
                Symbol::Terminal { kind } => return kind.generate(rng),
            }
        }

        if let Some(children) = &tree.children {
            if !ctx.regenerated {
                let new_children: Vec<_> = children
                    .iter()
                    .map(|child| do_regenerate(lang, child.clone(), target, ctx, rng))
                    .collect();

                if children
                    .iter()
                    .zip(&new_children)
                    .any(|(old, new)| !Arc::ptr_eq(old, new))
                {
                    return Arc::new(DerivationTree {
                        symbol: tree.symbol.clone(),
                        children: Some(new_children),
                    });
                }
            }
        }

        tree
    }

    let mut ctx = RegenerateContext {
        position: 0,
        regenerated: false,
    };
    do_regenerate(lang, tree, target, &mut ctx, rng)
}

impl<S> Mutator<PangInput, S> for RegenerateFragmentMutator
where
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error> {
        if !input.has_structure {
            let pang_state = state.metadata_mut::<PangMutateState>().unwrap();
            if pang_state.add_seen_seed(input) {
                PangHelper::add_to_fragment_pool(&self.lang, pang_state, input);
            }
            if !input.has_structure {
                return Ok(MutationResult::Skipped);
            }
        }

        let tree = match &input.structure {
            Some(tree) => tree,
            None => return Ok(MutationResult::Skipped),
        };

        let n_nodes = PangHelper::count_nodes(&self.lang, tree);

        if n_nodes <= 1 {
            return Ok(MutationResult::Skipped);
        }

        let rng = state.rand_mut();

        let target_node_idx = rng.between(1, n_nodes);

        let new_tree = regenerate_node_at(&self.lang, tree.clone(), target_node_idx, rng);

        let new_bytes = new_tree.to_bytes();
        if new_bytes.is_empty() || new_bytes == input.bytes {
            return Ok(MutationResult::Skipped);
        }

        input.bytes = new_bytes;
        input.structure = Some(new_tree);
        input.has_regions = false;
        // input.regions = None;

        Ok(MutationResult::Mutated)
    }

    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
