//! Parser can parse input string with given grammar.

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{grammar::Grammar, symbol::Symbol, tree::DerivationTree};

pub mod combinator;
pub mod expansion;
pub mod factory;

/// Region contains a range of positions that is partially parsed.
#[derive(Debug, Clone, Copy, Eq)]
pub struct Region {
    /// Start position of the region.
    pub start: usize,
    /// End position of the region.
    pub end: usize,
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

/// Parser trait defines some common methods.
pub trait Parser: Debug {
    /// Returns the grammar.
    fn grammar(&self) -> &Grammar;

    /// Returns the start symbol.
    fn start_symbol(&self) -> &str;

    /// A set of non-terminal labels that should be treated as opaque tokens.
    /// When pruning, the entire subtree for such a token will be replaced by its string value.
    fn tokens(&self) -> &HashSet<String>;

    /// If true, adjacent terminal nodes in the derivation tree will be merged.
    fn coalesce_tokens(&self) -> bool;

    /// Parses input text and returns the first derivation tree.
    fn parse_first(&self, text: &[u8]) -> Result<Arc<DerivationTree>, String>;

    /// Parses input text and returns all possible derivation trees.
    fn parse_forest(&self, text: &[u8]) -> Result<Vec<Arc<DerivationTree>>, String>;

    /// If input can not be fully parsed, parser is able to give a regions map.
    ///`parse_regions` will find all parsable regions for all non-terminals in the grammar.
    fn parse_regions(&self, _text: &[u8]) -> Result<HashMap<String, HashSet<Region>>, String> {
        Ok(HashMap::new())
    }

    /// Returns true if a fragment starting with a specific
    /// symbol and all its decendents can be excluded
    fn is_excluded(&self, symbol: &Symbol) -> bool {
        match symbol {
            Symbol::Terminal { .. } => true,
            Symbol::NonTerminal { label } => {
                !self.grammar().contains_key(label) || self.tokens().contains(label)
            }
        }
    }
}
