use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    sync::{Arc, Mutex},
};

use nom::IResult;

use crate::{grammar::Expansion, tree::DerivationTree};

pub mod ber_length;

/// Result of custom parser.
#[derive(Debug)]
pub enum CustomParseResult<'a> {
    /// Parse success.
    Success((&'a [u8], Arc<DerivationTree>)),

    /// Do some Pre-treatment or inspection, and continue parsing with default logic.
    Continue,

    /// Custom parse failure.
    Failure,
}

/// User-defined parser trait.
pub trait CustomParser: Debug + Send + Sync {
    /// Parse the input data and return a list of derivation trees.
    fn parse<'a>(
        &self,
        input: &'a [u8],
        label: &str,
        expansion: &Expansion,
        context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> IResult<&'a [u8], Vec<Arc<DerivationTree>>>;
}

/// A factory function that creates a new parser instance.
pub type ParserFactory = Arc<dyn Fn(SharedState) -> Box<dyn CustomParser> + Send + Sync>;

/// A registry of custom parsers.
pub type ParserRegistry = HashMap<String, ParserFactory>;
