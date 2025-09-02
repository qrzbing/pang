//!

use std::collections::BTreeMap;
use std::sync::Arc;

use log::debug;

use crate::{
    grammar::{Expansion, Grammar},
    symbol::{DecodeError, DecodeResult, SharedState, Symbol, nt},
    tree::{DerivationTree, new_node},
};

impl Grammar {
    /// Parse input string with given grammar using combinator parser.
    pub fn parse_combinator<'a>(
        &'a self,
        input: &'a [u8],
        start_symbol: &str,
    ) -> Result<Arc<DerivationTree>, DecodeError> {
        let start_node = nt(start_symbol);
        let mut context = BTreeMap::new();

        match start_node.parse(input, self, &mut context) {
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
    pub fn parse<'a>(
        &'a self,
        input: &'a [u8],
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<DerivationTree>> {
        debug!(
            "--> SYMBOL PARSE: Trying to parse symbol: {:?}, input_len: {}",
            self,
            input.len()
        );
        match self {
            // Parse NonTerminal
            Symbol::NonTerminal { label } => {
                let expansions = grammar
                    .get(label)
                    .ok_or(DecodeError::Invalid("Non-terminal not found in grammar"))?;

                debug!(
                    "    NT '{}': Found {} expansion(s)",
                    label,
                    expansions.len()
                );

                for (i, expansion) in expansions.iter().enumerate() {
                    debug!("    NT '{}': Trying expansion #{}", label, i);

                    let mut temp_context = context.clone();

                    if let Ok((remaining_input, children)) =
                        expansion.parse(input, grammar, &mut temp_context)
                    {
                        *context = temp_context;
                        let node = new_node(self.clone(), Some(children));
                        context.insert(label.clone(), node.clone());

                        debug!(
                            "<-- SYMBOL PARSE SUCCESS (NT '{}'), remaining_len: {}",
                            label,
                            remaining_input.len()
                        );

                        return Ok((remaining_input, node));
                    } else {
                        debug!("    NT '{}': Expansion #{} FAILED.", label, i);
                    }
                }
                debug!(
                    "<-- SYMBOL PARSE FAILED (NT '{}'): No expansion matched.",
                    label
                );
                Err(DecodeError::Invalid("No expansion matched for NonTerminal"))
            }
            // Parse Terminal
            Symbol::Terminal { kind } => {
                let (remaining_input, new_kind) =
                    kind.parse(input, &SharedState::new(), context)?;
                let new_symbol = Symbol::Terminal { kind: new_kind };
                let node = new_node(new_symbol, Some(vec![]));
                Ok((remaining_input, node))
            }
        }
    }
}

impl Expansion {
    /// Parse an Expansion, applying callback if it exists.
    pub fn parse<'a>(
        &'a self,
        input: &'a [u8],
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Vec<Arc<DerivationTree>>> {
        // Helper function to parse the sequence of symbols.
        fn parse_symbols<'b>(
            input_slice: &'b [u8],
            expansion: &'b Expansion,
            grammar: &'b Grammar,
            context: &mut BTreeMap<String, Arc<DerivationTree>>,
        ) -> DecodeResult<'b, Vec<Arc<DerivationTree>>> {
            let mut remaining_input = input_slice;
            let mut children = Vec::new();
            for symbol in &expansion.symbols {
                match symbol.parse(remaining_input, grammar, context) {
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
            let (remaining_after_slice, data_for_expansion) = callback(input, context)?;
            let (rem_in_slice, children) =
                parse_symbols(&data_for_expansion, self, grammar, context)?;
            // The preprocessed data must be consumed entirely.
            if !rem_in_slice.is_empty() {
                return Err(DecodeError::Invalid(
                    "Expansion did not consume the entire slice from decode_callback",
                ));
            }
            Ok((remaining_after_slice, children))
        } else {
            // Default behavior: parse the input directly.
            parse_symbols(input, self, grammar, context)
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
