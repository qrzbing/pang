//! A Language can be defined by its grammar, start symbol, and other things.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    grammar::Grammar,
    parser::Region,
    symbol::{DecodeError, Symbol, terminals::DynRand},
    tree::{DerivationTree, TreeFixer},
};

// #[doc(hidden)]
pub mod examples;
pub use examples::{
    ASCII_LETTERS, DIGITS, c_sample_lang, expr_lang, srange, tlv::asn1_tlv_lang, xml::xml_lang,
};

/// Specify a Language.
#[derive(Clone, Debug)]
pub struct Language {
    /// Grammar of the language.
    pub grammar: Grammar,
    /// Start symbol
    pub start_symbol: String,
    /// A set of non-terminal labels that should be treated as opaque tokens.
    /// When pruning, the entire subtree for such a token will be replaced by its string value.
    pub tokens: HashSet<String>,
}

impl Language {
    /// Create a new Language.
    pub fn new(grammar: Grammar, start_symbol: &str, tokens: HashSet<String>) -> Self {
        Self {
            grammar,
            start_symbol: start_symbol.to_string(),
            tokens,
        }
    }

    /// Returns true if a fragment starting with a specific symbol and
    /// all its decendents can be excluded.
    pub fn is_excluded(&self, symbol: &Symbol) -> bool {
        match symbol {
            Symbol::Terminal { .. } => true,
            Symbol::NonTerminal { label } => {
                !self.grammar.contains_key(label) || self.tokens.contains(label)
            }
        }
    }

    /// Parse an input bytes slice to a derivation tree.
    pub fn parse<'a>(&'a self, input: &'a [u8]) -> Result<Arc<DerivationTree>, DecodeError> {
        self.grammar.parse_combinator(input, &self.start_symbol)
    }

    /// Generate random input based on [`Grammar`].
    pub fn generate(
        &self,
        rng: &mut dyn DynRand,
        fixers: &[Arc<dyn TreeFixer>],
    ) -> Arc<DerivationTree> {
        self.grammar
            .generate_combinator(&self.start_symbol, rng, fixers)
    }

    /// TODO: Once input can not be fully parsed, collect its regions and return.
    pub fn parse_regions(&self, _input: &[u8]) -> Result<HashMap<String, HashSet<Region>>, String> {
        todo!("not implemented yet")
    }
}
