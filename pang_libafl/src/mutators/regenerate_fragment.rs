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

use pang::{
    generator::Generator,
    grammar::{Symbol, TerminalKind},
    parser::Parser,
    tree::DerivationTree,
};

use crate::{input::PangInput, mutators::PangHelper, state::PangMutateState};

/// # RegenerateFragmentMutator
///
/// Select a random node and regenerate its subtree.
#[derive(Debug)]
pub struct RegenerateFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    parser: Arc<Mutex<P>>,
    generator: Arc<Mutex<Generator>>,
}

impl<P> RegenerateFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    /// Creates a new [`RegenerateFragmentMutator`].
    #[must_use]
    pub fn new(parser: Arc<Mutex<P>>, generator: Arc<Mutex<Generator>>) -> Self {
        Self { parser, generator }
    }
}

impl<P> Named for RegenerateFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
{
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("RegenerateFragmentMutator")
    }
}

fn regenerate_node_at<P: Parser>(
    parser: &P,
    generator: &Generator,
    tree: Arc<DerivationTree>,
    target: usize,
) -> Arc<DerivationTree> {
    struct RegenerateContext {
        position: usize,
        regenerated: bool,
    }

    fn do_regenerate<P: Parser>(
        parser: &P,
        generator: &Generator,
        tree: Arc<DerivationTree>,
        target: usize,
        ctx: &mut RegenerateContext,
    ) -> Arc<DerivationTree> {
        // Skip excluded nodes
        if parser.is_excluded(&tree.symbol) {
            return tree;
        }

        ctx.position += 1;

        if ctx.position == target {
            ctx.regenerated = true;
            // For non-terminals, generate a new subtree
            match &tree.symbol {
                Symbol::NonTerminal { label } => {
                    return generator.generate_from_symbol(label);
                }
                Symbol::Terminal { kind } => match kind {
                    TerminalKind::Binary(_bin_kind) => {
                        // TODO: Regenerate binary values
                        return tree;
                    }
                    TerminalKind::Literal(..) => return tree,
                },
            }
        }

        if let Some(children) = &tree.children {
            if !ctx.regenerated {
                let new_children: Vec<_> = children
                    .iter()
                    .map(|child| do_regenerate(parser, generator, child.clone(), target, ctx))
                    .collect();

                if children
                    .iter()
                    .zip(&new_children)
                    .any(|(old, new)| !Arc::ptr_eq(old, new))
                {
                    return Arc::new(DerivationTree {
                        symbol: tree.symbol.clone(),
                        children: Some(new_children),
                        value: None,
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
    do_regenerate(parser, generator, tree, target, &mut ctx)
}

impl<P, S> Mutator<PangInput, S> for RegenerateFragmentMutator<P>
where
    P: Parser + Send + Sync + 'static,
    S: HasRand + HasMetadata,
{
    fn mutate(&mut self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error> {
        if !input.has_structure {
            let pang_state = state.metadata_mut::<PangMutateState>().unwrap();
            if pang_state.add_seen_seed(input) {
                PangHelper::add_to_fragment_pool(self.parser.clone(), pang_state, input);
            }
            if !input.has_structure {
                return Ok(MutationResult::Skipped);
            }
        }

        let tree = match &input.structure {
            Some(tree) => tree,
            None => return Ok(MutationResult::Skipped),
        };

        let parser = self.parser.lock().unwrap();
        let n_nodes = PangHelper::count_nodes(&*parser, tree);

        if n_nodes <= 1 {
            return Ok(MutationResult::Skipped);
        }

        let target_node_idx = state.rand_mut().between(1, n_nodes);

        let generator = self.generator.lock().unwrap();

        let new_tree = regenerate_node_at(&*parser, &*generator, tree.clone(), target_node_idx);

        let new_bytes = new_tree.to_bytes();
        if new_bytes.is_empty() || new_bytes == input.bytes {
            return Ok(MutationResult::Skipped);
        }

        input.bytes = new_bytes;
        input.structure = Some(new_tree);
        input.has_regions = false;
        input.regions = None;

        Ok(MutationResult::Mutated)
    }

    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
