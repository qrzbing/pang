//! Format Parser is for context-free grammar like TLV etc.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex},
};

use nom::{
    Err, IResult, Needed,
    error::{ErrorKind, ParseError},
};

use crate::{
    grammar::{Expansion, Grammar, Symbol, TerminalKind, nt, symbol::BinaryKind},
    parser::{Parser, Region},
    tree::{DerivationTree, new_node},
};

mod custom;
pub use custom::{
    CustomParseResult, CustomParser, ParserFactory, ParserRegistry, SharedState,
    ber_length::BerLengthParser,
};
mod logic;
mod terminal;

/// Convert bytes to usize.
///
/// # Examples
///
/// ```
/// use serde_json::json;
///
/// use pang::parser::format::bytes_to_usize;
///
/// assert_eq!(bytes_to_usize(b"\x01\x00\x00\x00\x00\x00\x00\x00", Some(&json!("little"))), 1);
/// assert_eq!(bytes_to_usize(b"\x01\x00\x00\x00", Some(&json!("little"))), 1);
/// assert_eq!(bytes_to_usize(b"\x01\x00", Some(&json!("little"))), 1);
/// assert_eq!(bytes_to_usize(b"\x01", Some(&json!("little"))), 1);
/// assert_eq!(bytes_to_usize(b"\x00\x00\x00\x00\x00\x00\x00\x01", Some(&json!("big"))), 1);
/// assert_eq!(bytes_to_usize(b"\x00\x00\x00\x01", Some(&json!("big"))), 1);
/// assert_eq!(bytes_to_usize(b"\x00\x01", Some(&json!("big"))), 1);
/// assert_eq!(bytes_to_usize(b"\x01", Some(&json!("big"))), 1);
/// ```
pub fn bytes_to_usize(bytes: &[u8], endian: Option<&serde_json::Value>) -> usize {
    let is_little = endian
        .and_then(|v| v.as_str())
        .map_or(false, |s| s == "little");

    let mut buf = [0u8; 8];
    let len = bytes.len().min(8);

    if is_little {
        buf[..len].copy_from_slice(&bytes[..len]);
        usize::from_le_bytes(buf)
    } else {
        buf[8 - len..].copy_from_slice(&bytes[..len]);
        usize::from_be_bytes(buf)
    }
}

/// Parse a BER-encoded length field.
pub fn parse_ber_length_field(input: &[u8]) -> IResult<&[u8], &[u8]> {
    // Ensure we have at least one byte to read.
    if input.is_empty() {
        return Err(Err::Incomplete(Needed::new(1)));
    }

    let first_byte = input[0];
    let field_len = if (first_byte & 0x80) == 0 {
        // Short form: the field is exactly 1 byte long.
        1
    } else {
        // Long form: the total length is 1 (for the first byte)
        // plus the number of bytes indicated in the lower 7 bits.
        let num_len_bytes = (first_byte & 0x7F) as usize;
        1 + num_len_bytes
    };

    // Check if we have enough bytes in the input for the full field.
    if input.len() < field_len {
        return Err(Err::Incomplete(Needed::new(field_len - input.len())));
    }

    // Split the input at the calculated field length.
    Ok((&input[field_len..], &input[..field_len]))
}

/// Convert a BER-encoded length field to a usize.
pub fn ber_to_usize(bytes: &[u8]) -> Result<usize, &'static str> {
    if bytes.is_empty() {
        return Err("BER length field cannot be empty");
    }

    let first_byte = bytes[0];
    if (first_byte & 0x80) == 0 {
        // In short form, the length is the byte itself.
        // The field must be exactly one byte long.
        if bytes.len() > 1 {
            Err("Invalid BER short form: length field is longer than 1 byte")
        } else {
            Ok(first_byte as usize)
        }
    } else {
        // Long form (MSB is 1)
        let num_len_bytes = (first_byte & 0x7F) as usize;

        if num_len_bytes == 0 {
            return Err("Invalid BER long form: number of length bytes cannot be zero");
        }

        // Check if the actual number of bytes matches the number advertised.
        // The total length of the slice should be 1 (for the initial byte) + num_len_bytes.
        if bytes.len() != 1 + num_len_bytes {
            return Err("Invalid BER long form: mismatched number of length bytes");
        }

        let len_bytes = &bytes[1..];
        let mut length: usize = 0;
        for &byte in len_bytes {
            // Manual big-endian conversion
            length = (length << 8) + (byte as usize);
        }
        Ok(length)
    }
}

/// Collect different solutions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseMode {
    /// Return the first parsing tree.
    First,
    /// Return all parsing trees.
    Forest,
    /// Collect Regions
    Region,
}

/// Format Parser contains some methods for binary format grammar parsing.
#[derive(Debug)]
pub struct FormatParser {
    grammar: Grammar,
    start_symbol: String,
    tokens: HashSet<String>,
    coalesce_tokens: bool,
    regions: Mutex<HashMap<String, HashSet<Region>>>,
    original_input_ptr: Mutex<usize>,
    // Custom parsers
    custom_parsers: HashMap<String, Box<dyn CustomParser>>,
    // Share State
    state: SharedState,
}

impl Parser for FormatParser {
    /// Return first successful parse of the input text.
    fn parse_first(&self, text: &[u8]) -> Result<Arc<DerivationTree>, String> {
        match self.parse_non_terminal_with_mode(
            text,
            &self.start_symbol,
            &BTreeMap::new(),
            ParseMode::First,
        ) {
            Ok((_remaining_input, trees)) => {
                let (remaining_input, tree) = &trees[0];
                if !remaining_input.is_empty() {
                    Err(format!(
                        "Input was not fully consumed. {} bytes remaining.",
                        remaining_input.len()
                    ))
                } else {
                    Ok(Arc::clone(tree))
                }
            }
            Err(e) => Err(format!("No successful parse found: {}", e)),
        }
    }

    /// Return all successful parse of the input text.
    fn parse_forest(&self, text: &[u8]) -> Result<Vec<Arc<DerivationTree>>, String> {
        match self.parse_non_terminal_with_mode(
            text,
            &self.start_symbol,
            &BTreeMap::new(),
            ParseMode::Forest,
        ) {
            Ok((_remaining_input, trees)) => {
                let mut best_trees = vec![];
                let mut min_remaining = usize::MAX;

                for (rem, tree) in trees {
                    let rem_len = rem.len();
                    if rem_len < min_remaining {
                        min_remaining = rem_len;
                        best_trees = vec![tree];
                    } else if rem_len == min_remaining {
                        best_trees.push(tree);
                    }
                }

                if min_remaining > 0 {
                    Err(format!(
                        "Input was not fully consumed. {} bytes remaining.",
                        min_remaining
                    ))
                } else if best_trees.is_empty() {
                    Err("No successful parse found.".to_string())
                } else {
                    Ok(best_trees)
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn parse_regions(&self, text: &[u8]) -> Result<HashMap<String, HashSet<Region>>, String> {
        self.parse_and_collect_regions(text)
    }

    fn grammar(&self) -> &Grammar {
        &self.grammar
    }

    fn start_symbol(&self) -> &str {
        &self.start_symbol
    }

    fn tokens(&self) -> &HashSet<String> {
        &self.tokens
    }
    fn coalesce_tokens(&self) -> bool {
        self.coalesce_tokens
    }
}

impl FormatParser {
    /// Create a new Format Parser with the given grammar and start symbol.
    pub fn new(grammar: Grammar, start_symbol: &str, parsers_registry: &ParserRegistry) -> Self {
        let shared_state = SharedState::new();

        let mut parser = FormatParser {
            grammar: grammar,
            start_symbol: start_symbol.to_string(),
            tokens: HashSet::new(),
            coalesce_tokens: false,
            regions: Mutex::new(HashMap::new()),
            original_input_ptr: Mutex::new(0),
            custom_parsers: HashMap::new(),
            state: shared_state.clone(),
        };

        for (label, factory) in parsers_registry {
            parser.register_parser(label, factory(shared_state.clone()));
        }

        parser
    }

    /// Register a custom parser for a given label.
    pub fn register_parser(&mut self, label: &str, parser: Box<dyn CustomParser>) {
        self.custom_parsers.insert(label.to_string(), parser);
    }

    fn extract_value_from_node(&self, node: &Arc<DerivationTree>) -> Vec<u8> {
        if let Some(ref value) = node.value {
            return value.clone();
        }

        if let Some(ref children) = node.children {
            for child in children {
                if let Some(ref value) = child.value {
                    return value.clone();
                }
            }
        }

        vec![]
    }
}
