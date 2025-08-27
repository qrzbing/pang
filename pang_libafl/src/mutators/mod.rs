//! LibAFL mutators based on Pang.

use std::{
    borrow::Cow,
    sync::{
        Arc, Mutex,
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::Duration,
};

use libafl::{
    Error, HasMetadata,
    corpus::CorpusId,
    inputs::BytesInput,
    mutators::{
        BitFlipMutator, ByteAddMutator, ByteDecMutator, ByteFlipMutator, ByteIncMutator,
        ByteInterestingMutator, ByteNegMutator, ByteRandMutator, BytesCopyMutator,
        BytesDeleteMutator, BytesExpandMutator, BytesInsertCopyMutator, BytesInsertMutator,
        BytesRandInsertMutator, BytesRandSetMutator, BytesSetMutator, BytesSwapMutator,
        DwordAddMutator, DwordInterestingMutator, MutationResult, Mutator, QwordAddMutator,
        WordAddMutator, WordInterestingMutator,
    },
    state::{HasMaxSize, HasRand},
};
use libafl_bolts::{
    Named,
    tuples::{tuple_list, tuple_list_type},
};
use log::{debug, warn};

use pang::{generator::Generator, parser::Parser, symbol::Symbol, tree::DerivationTree};

use crate::{input::PangInput, state::PangMutateState};

mod add_fragment;
pub use add_fragment::AddFragmentMutator;
mod delete_fragment;
pub use delete_fragment::DeleteFragmentMutator;
mod regenerate_fragment;
pub use regenerate_fragment::RegenerateFragmentMutator;
mod swap_fragment;
pub use swap_fragment::SwapFragmentMutator;

/// A helper struct for Pang mutators, perhaps need to refactor later.
#[derive(Debug)]
pub struct PangHelper;

impl PangHelper {
    /// Add fragments
    ///
    /// A fragment is a subtree in the parse tree and consists of the symbol
    /// of the current node and child nodes (i.e., descendant fragments).
    fn add_fragment<P: Parser>(
        parser: &P,
        pang_state: &mut PangMutateState,
        fragment: Arc<DerivationTree>,
    ) {
        let symbol = &fragment.symbol;
        if !parser.is_excluded(symbol) {
            if let Symbol::NonTerminal { label } = symbol {
                pang_state
                    .fragments
                    .get_mut(label)
                    .expect("Fragment key should exist for non-excluded symbols")
                    .push(fragment.clone());

                if let Some(children) = &fragment.children {
                    for subfragment in children {
                        Self::add_fragment(parser, pang_state, subfragment.clone());
                    }
                }
            }
        }
    }

    /// Parses a seed (no longer than 200ms) and adds all its fragments to the fragment pool.
    /// If the parsing of the seed was successful, the attribute seed.has_structure is set to True.
    /// Otherwise, it is set to False.
    pub fn add_to_fragment_pool<P>(
        parser: Arc<Mutex<P>>,
        pang_state: &mut PangMutateState,
        input: &mut PangInput,
    ) where
        P: Parser + Send + Sync + 'static,
    {
        debug!("Parse for structure");
        // Parse seed for structure
        let (tx, rx) = mpsc::channel();
        let timeout = Duration::from_millis(200);
        let parser_for_thread = Arc::clone(&parser);
        let bytes_to_parse = input.bytes.clone();
        thread::spawn(move || {
            let parser_locked = parser_for_thread.lock().unwrap();
            if let Ok(tree) = parser_locked.parse_first(&bytes_to_parse) {
                let _ = tx.send(tree);
            }
        });
        match rx.recv_timeout(timeout) {
            Ok(tree) => {
                let parser_locked = parser.lock().unwrap();
                Self::add_fragment(&*parser_locked, pang_state, tree.clone());
                input.has_structure = true;
                input.structure = Some(tree);
            }
            Err(RecvTimeoutError::Timeout) => {
                warn!("Parsing timed out.");
                input.has_structure = false;
                input.structure = None;
            }
            Err(RecvTimeoutError::Disconnected) => {
                warn!("Parsing failed or produced no result.");
                input.has_structure = false;
                input.structure = None;
            }
        };

        // Do partial parsing for regions
        if !input.has_structure {
            debug!("No structure for seed, parse regions instead.");
            let (tx, rx) = mpsc::channel();
            let timeout = Duration::from_millis(200);
            let parser_for_thread = Arc::clone(&parser);
            let bytes_to_parse = input.bytes.clone();
            thread::spawn(move || {
                let parser_locked = parser_for_thread.lock().unwrap();

                if let Ok(regions) = parser_locked.parse_regions(&bytes_to_parse) {
                    let _ = tx.send(regions);
                }
            });
            match rx.recv_timeout(timeout) {
                Ok(regions) => {
                    input.has_regions = true;
                    input.regions = Some(regions);
                }
                Err(RecvTimeoutError::Timeout) => {
                    warn!("Parsing timed out.");
                    input.has_regions = false;
                    input.regions = None;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    warn!("Parsing failed or produced no result.");
                    input.has_regions = false;
                    input.regions = None;
                }
            };
        }
    }

    /// In order to choose a random fragment, the mutator counts all fragments (n_count)
    /// below the root fragment associated with the start-symbol.
    fn count_nodes<P: Parser>(parser: &P, tree: &DerivationTree) -> usize {
        if parser.is_excluded(&tree.symbol) {
            return 0;
        }

        let children_count: usize = tree.children.as_ref().map_or(0, |children| {
            children.iter().map(|c| Self::count_nodes(parser, c)).sum()
        });

        1 + children_count
    }
}

/// Havoc mutations with both byte-level and grammar-level mutators.
pub fn havoc_mutations_pang<S, P>(
    generator: Arc<Mutex<Generator>>,
    parser: Arc<Mutex<P>>,
) -> tuple_list_type!(
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    // For PangInput
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
)
where
    S: HasRand + 'static + HasMaxSize + HasMetadata,
    P: Parser + Send + Sync + 'static,
{
    tuple_list!(
        PangMutator::from_bytes_mutator(Box::new(BitFlipMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteAddMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteDecMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteFlipMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteIncMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteInterestingMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteNegMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(ByteRandMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesCopyMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesDeleteMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesExpandMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesInsertCopyMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesInsertMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesRandInsertMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesRandSetMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesSetMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(BytesSwapMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(DwordAddMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(DwordInterestingMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(QwordAddMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(WordAddMutator::new())),
        PangMutator::from_bytes_mutator(Box::new(WordInterestingMutator::new())),
        // For PangInput
        PangMutator::from_series_mutator(Box::new(AddFragmentMutator::new(parser.clone()))),
        PangMutator::from_series_mutator(Box::new(DeleteFragmentMutator::new(parser.clone()))),
        PangMutator::from_series_mutator(Box::new(SwapFragmentMutator::new(parser.clone()))),
        PangMutator::from_series_mutator(Box::new(RegenerateFragmentMutator::new(
            parser.clone(),
            generator.clone()
        ))),
    )
}

/// A set of Pang mutators.
pub fn pang_mutations<S, P>(
    generator: Arc<Mutex<Generator>>,
    parser: Arc<Mutex<P>>,
) -> tuple_list_type!(
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
    PangMutator<S>,
)
where
    S: HasRand + 'static + HasMaxSize + HasMetadata,
    P: Parser + Send + Sync + 'static,
{
    tuple_list!(
        PangMutator::from_series_mutator(Box::new(AddFragmentMutator::new(parser.clone()))),
        PangMutator::from_series_mutator(Box::new(DeleteFragmentMutator::new(parser.clone()))),
        PangMutator::from_series_mutator(Box::new(SwapFragmentMutator::new(parser.clone()))),
        PangMutator::from_series_mutator(Box::new(RegenerateFragmentMutator::new(
            parser.clone(),
            generator.clone()
        ))),
    )
}

/// Pang mutator can wrap both byte-level mutators and grammar-level mutators.
pub enum PangMutator<S>
where
    S: HasRand,
{
    /// Mutator that manipulates the contents of one request in a chain
    Contents(Box<dyn Mutator<BytesInput, S>>),
    /// Mutator that manipulates the order or number of request in the chain
    Series(Box<dyn Mutator<PangInput, S>>),
}

impl<S> std::fmt::Debug for PangMutator<S>
where
    S: HasRand,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contents(mutator) => f.debug_tuple("Contents").field(&mutator.name()).finish(),
            Self::Series(mutator) => f.debug_tuple("Series").field(&mutator.name()).finish(),
        }
    }
}

impl<S> Named for PangMutator<S>
where
    S: HasRand,
{
    fn name(&self) -> &Cow<'static, str> {
        &Cow::Borrowed("PangMutator")
    }
}

impl<S> PangMutator<S>
where
    S: HasRand,
{
    /// Creates a new request-contents-mutator given a BytesInput mutator
    #[must_use]
    pub fn from_bytes_mutator(mutator: Box<dyn Mutator<BytesInput, S>>) -> Self {
        Self::Contents(mutator)
    }

    /// Creates a new request-series-mutator given an PangInput mutator
    #[must_use]
    pub fn from_series_mutator(mutator: Box<dyn Mutator<PangInput, S>>) -> Self {
        Self::Series(mutator)
    }
}

impl<S> Mutator<PangInput, S> for PangMutator<S>
where
    S: HasRand,
{
    fn mutate(&mut self, state: &mut S, input: &mut PangInput) -> Result<MutationResult, Error> {
        match self {
            PangMutator::Contents(contents_mutator) => {
                let mut temp_bytes_input = BytesInput::from(std::mem::take(&mut input.bytes));
                let result = contents_mutator.mutate(state, &mut temp_bytes_input)?;
                input.bytes = temp_bytes_input.into_inner();
                if result == MutationResult::Mutated {
                    input.structure = None;
                    input.has_structure = false;
                    input.regions = None;
                    input.has_regions = false;
                }
                Ok(result)
            }
            PangMutator::Series(b) => b.mutate(state, input),
        }
    }

    #[inline]
    fn post_exec(&mut self, _state: &mut S, _new_corpus_id: Option<CorpusId>) -> Result<(), Error> {
        Ok(())
    }
}
