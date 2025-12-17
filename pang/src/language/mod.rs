//! A Language can be defined by its grammar, start symbol, and other things.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[allow(deprecated)]
use crate::{
    EnumMapping, ParseState,
    grammar::Grammar,
    parser::Region,
    symbol::{DecodeError, Symbol},
    tree::DerivationTree,
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
    /// EnumMapping for SwitchNonTerminal
    pub enums: Arc<HashMap<String, EnumMapping>>,
}

impl Language {
    /// Create a new Language.
    pub fn new(
        grammar: &Grammar,
        start_symbol: &str,
        tokens: HashSet<String>,
        enums: HashMap<String, EnumMapping>,
    ) -> Self {
        let enums = Arc::new(enums);
        assert!(grammar.is_valid(start_symbol, &enums));
        Self {
            grammar: grammar.clone(),
            start_symbol: start_symbol.to_string(),
            tokens,
            enums,
        }
    }

    /// Returns true if a fragment starting with a specific symbol and
    /// all its decendents can be excluded.
    pub fn is_excluded(&self, symbol: &Symbol) -> bool {
        match symbol {
            Symbol::Terminal { .. } => true,
            Symbol::NonTerminal { kind } => {
                !self.grammar.contains_key(kind.label()) || self.tokens.contains(kind.label())
            }
        }
    }

    /// Parse an input bytes slice to a derivation tree.
    #[allow(deprecated)]
    pub fn parse<'a>(&'a self, input: &'a [u8]) -> Result<Arc<DerivationTree>, DecodeError> {
        let mut state = ParseState::new(self.enums.clone());

        self.grammar
            .parse_combinator(&mut state, input, &self.start_symbol)
    }

    /// TODO: Once input can not be fully parsed, collect its regions and return.
    pub fn parse_regions(&self, _input: &[u8]) -> Result<HashMap<String, HashSet<Region>>, String> {
        todo!("not implemented yet")
    }
}
