//!

use std::collections::BTreeMap;
use std::sync::Arc;

use log::debug;

#[allow(deprecated)]
use crate::{
    ParseState,
    grammar::{Expansion, Grammar},
    symbol::{DecodeError, DecodeResult, SharedState, Symbol, nt},
    tree::{DerivationTree, new_node},
};

impl Grammar {
    /// Parse input string with given grammar using combinator parser.
    #[allow(unused_variables)]
    #[deprecated]
    #[allow(deprecated)]
    pub fn parse_combinator<'a>(
        &'a self,
        state: &mut ParseState,
        input: &'a [u8],
        start_symbol: &str,
    ) -> Result<Arc<DerivationTree>, DecodeError> {
        let start_node = nt(start_symbol);
        state.context = BTreeMap::new();

        match start_node.parse(state, input, self) {
            Ok((remaining, tree)) => {
                if !remaining.is_empty() {
                    debug!(
                        "Warning: Input not fully consumed. {} bytes remaining.",
                        remaining.len()
                    );
                }
                Ok(tree)
            }
            Err(e) => Err(e),
        }
    }
}

impl Symbol {
    /// Parse a Symbol.
    #[allow(unused_variables)]
    #[deprecated]
    #[allow(deprecated)]
    pub fn parse<'a, 'i>(
        &'a self,
        state: &mut ParseState,
        input: &'i [u8],
        grammar: &'i Grammar,
    ) -> DecodeResult<'i, Arc<DerivationTree>> {
        debug!(
            "--> SYMBOL PARSE: Trying to parse symbol: {:?}, input_len: {}",
            self,
            input.len()
        );
        match self {
            // Parse NonTerminal
            Symbol::NonTerminal { kind } => Ok(kind.parse(state, input, grammar)?),
            // Parse Terminal
            Symbol::Terminal { label, kind } => {
                let (remaining_input, new_kind) = kind.parse(input, &SharedState::new())?;
                let new_symbol = Symbol::Terminal {
                    label: label.clone(),
                    kind: new_kind,
                };
                let node = new_node(new_symbol, Some(vec![]));
                // Insert label into context if it exists.
                if let Some(lbl) = label {
                    state.context.insert(lbl.clone(), node.clone());
                }
                Ok((remaining_input, node))
            }
        }
    }
}

impl Expansion {
    /// Parse an Expansion, applying callback if it exists.
    #[deprecated]
    #[allow(deprecated)]
    pub fn parse<'a, 'i>(
        &'a self,
        state: &mut ParseState,
        input: &'i [u8],
        grammar: &'i Grammar,
    ) -> DecodeResult<'i, Vec<Arc<DerivationTree>>>
    where
        'a: 'i,
    {
        // Helper function to parse the sequence of symbols.
        fn parse_symbols<'b, 'g>(
            state: &mut ParseState,
            input_slice: &'b [u8],
            expansion: &'g Expansion,
            grammar: &'g Grammar,
        ) -> DecodeResult<'b, Vec<Arc<DerivationTree>>>
        where
            'g: 'b,
        {
            let mut remaining_input = input_slice;
            let mut children = Vec::new();
            for symbol in &expansion.symbols {
                match symbol.parse(state, remaining_input, grammar) {
                    Ok((next_input, child_node)) => {
                        remaining_input = next_input;
                        children.push(child_node);
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok((remaining_input, children))
        }
        if let Some(callback) = self.decode_callback {
            let (remaining_after_slice, data_for_expansion) = callback(input, &state.context)?;
            let (rem_in_slice, children) =
                parse_symbols(state, &data_for_expansion, self, grammar)?;
            // The preprocessed data must be consumed entirely.
            if !rem_in_slice.is_empty() {
                return Err(DecodeError::Invalid(
                    "Expansion did not consume the entire slice from decode_callback".into(),
                ));
            }
            Ok((remaining_after_slice, children))
        } else {
            // Default behavior: parse the input directly.
            parse_symbols(state, input, self, grammar)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use crate::language::{
        asn1_tlv_lang,
        examples::tlv::{nest_tlv_lang, tlv_lang},
    };

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            let _ = env_logger::try_init();
        });
    }

    #[test]
    fn test_parse_tlv_lang() {
        setup_logger();
        let input = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08];
        let lang = tlv_lang();
        let tree = lang.parse(input).unwrap();
        assert_eq!(tree.to_bytes(), input);
    }

    #[test]
    fn test_parse_nest_tlv_lang() {
        setup_logger();
        let lang = nest_tlv_lang();
        let input = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08];
        let tree = lang.parse(input).unwrap();
        assert_eq!(tree.to_bytes(), input);

        let input = &[
            0x01, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x09, // length
            0x01, 0x00, 0x00, 0x00, // nest-type
            0x00, 0x00, 0x00, 0x01, // nest-length
            0x01, // nest-value
        ];

        let tree = lang.parse(input).unwrap();
        assert_eq!(tree.to_bytes(), input);

        let input = &[
            0x01, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x18, // length
            0x01, 0x00, 0x00, 0x00, // nest-type
            0x00, 0x00, 0x00, 0x10, // nest-length
            0x01, 0x00, 0x00, 0x00, // nest-nest-type
            0x00, 0x00, 0x00, 0x08, // nest-nest-length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // nest-nest-value
        ];
        let tree = lang.parse(input).unwrap();
        assert_eq!(tree.to_bytes(), input);
    }

    #[test]
    fn test_parse_asn1_lang() {
        setup_logger();
        let lang = asn1_tlv_lang();

        let input = &[0x02, 0x01, 0x00];
        let tree = lang.parse(input).unwrap();
        assert_eq!(tree.to_bytes(), input);

        let input = &[0x05, 0x00];
        let tree = lang.parse(input).unwrap();
        assert_eq!(tree.to_bytes(), input);
    }
}
