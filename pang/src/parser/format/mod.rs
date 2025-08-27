//! Format Parser is for context-free grammar like TLV etc.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::{Arc, Mutex},
};

use crate::{
    grammar::{Expansion, Grammar},
    parser::{Parser, Region},
    symbol::SharedState,
    tree::{DerivationTree, new_node},
};

// mod custom;
// pub use custom::{
//     CustomParseResult, CustomParser, ParserFactory, ParserRegistry,
//     ber_length::BerLengthParser,
// };
mod logic;
mod terminal;

// /// Convert bytes to usize.
// ///
// /// # Examples
// ///
// /// ```
// /// use serde_json::json;
// ///
// /// use pang::parser::format::bytes_to_usize;
// ///
// /// assert_eq!(bytes_to_usize(b"\x01\x00\x00\x00\x00\x00\x00\x00", Some(&json!("little"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x01\x00\x00\x00", Some(&json!("little"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x01\x00", Some(&json!("little"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x01", Some(&json!("little"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x00\x00\x00\x00\x00\x00\x00\x01", Some(&json!("big"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x00\x00\x00\x01", Some(&json!("big"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x00\x01", Some(&json!("big"))), 1);
// /// assert_eq!(bytes_to_usize(b"\x01", Some(&json!("big"))), 1);
// /// ```
// pub fn bytes_to_usize(bytes: &[u8], endian: Option<&serde_json::Value>) -> usize {
//     let is_little = endian
//         .and_then(|v| v.as_str())
//         .map_or(false, |s| s == "little");

//     let mut buf = [0u8; 8];
//     let len = bytes.len().min(8);

//     if is_little {
//         buf[..len].copy_from_slice(&bytes[..len]);
//         usize::from_le_bytes(buf)
//     } else {
//         buf[8 - len..].copy_from_slice(&bytes[..len]);
//         usize::from_be_bytes(buf)
//     }
// }

// /// Parse a BER-encoded length field.
// pub fn parse_ber_length_field(input: &[u8]) -> IResult<&[u8], &[u8]> {
//     // Ensure we have at least one byte to read.
//     if input.is_empty() {
//         return Err(Err::Incomplete(Needed::new(1)));
//     }

//     let first_byte = input[0];
//     let field_len = if (first_byte & 0x80) == 0 {
//         // Short form: the field is exactly 1 byte long.
//         1
//     } else {
//         // Long form: the total length is 1 (for the first byte)
//         // plus the number of bytes indicated in the lower 7 bits.
//         let num_len_bytes = (first_byte & 0x7F) as usize;
//         1 + num_len_bytes
//     };

//     // Check if we have enough bytes in the input for the full field.
//     if input.len() < field_len {
//         return Err(Err::Incomplete(Needed::new(field_len - input.len())));
//     }

//     // Split the input at the calculated field length.
//     Ok((&input[field_len..], &input[..field_len]))
// }

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
    pub fn new(grammar: Grammar, start_symbol: &str) -> Self {
        let shared_state = SharedState::new();

        FormatParser {
            grammar: grammar,
            start_symbol: start_symbol.to_string(),
            tokens: HashSet::new(),
            coalesce_tokens: false,
            regions: Mutex::new(HashMap::new()),
            original_input_ptr: Mutex::new(0),
            state: shared_state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use crate::{
        grammar::{
            asn1_tlv_grammar,
            examples::tlv::{nest_tlv_grammar, tlv_grammar},
        },
        parser::{FormatParser, Parser},
    };

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            let _ = env_logger::try_init();
        });
    }

    #[test]
    fn test_tlv_grammar() {
        setup_logger();
        let parser = FormatParser::new(tlv_grammar(), "start");
        let input = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08];
        let result = parser.parse_first(input).unwrap();
        assert_eq!(result.to_bytes(), input);
    }

    #[test]
    fn test_nest_tlv_grammar() {
        setup_logger();
        let parser = FormatParser::new(nest_tlv_grammar(), "start");
        let input = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08];
        let result = parser.parse_first(input).unwrap();
        assert_eq!(result.to_bytes(), input);

        let input = &[
            0x01, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x09, // length
            0x01, 0x00, 0x00, 0x00, // nest-type
            0x00, 0x00, 0x00, 0x01, // nest-length
            0x01, // nest-value
        ];
        let result = parser.parse_first(input).unwrap();
        assert_eq!(result.to_bytes(), input);

        let input = &[
            0x01, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x18, // length
            0x01, 0x00, 0x00, 0x00, // nest-type
            0x00, 0x00, 0x00, 0x10, // nest-length
            0x01, 0x00, 0x00, 0x00, // nest-nest-type
            0x00, 0x00, 0x00, 0x08, // nest-nest-length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // nest-nest-value
        ];
        let result = parser.parse_first(input).unwrap();
        assert_eq!(result.to_bytes(), input);
    }

    #[test]
    fn test_asn1_grammar() {
        setup_logger();
        let parser = FormatParser::new(asn1_tlv_grammar(), "asn1-tlv");

        let input = &[0x02, 0x01, 0x00];
        let result = parser.parse_first(input).unwrap();
        assert_eq!(result.to_bytes(), input);

        let input = &[0x05, 0x00];
        let result = parser.parse_first(input).unwrap();
        assert_eq!(result.to_bytes(), input);
    }
}
