//!

use std::collections::BTreeMap;
use std::sync::Arc;

use log::debug;

use crate::{
    grammar::{Expansion, ExpansionCallback, Grammar},
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

                    let parse_result = if let Some(boxed_callback) =
                        expansion.options.get("length_calculator")
                    {
                        // Get length from callback
                        let length = if let Some(callback) =
                            boxed_callback.downcast_ref::<ExpansionCallback>()
                        {
                            (*callback)(&temp_context)?
                        } else {
                            return Err(DecodeError::Invalid(
                                "Option 'length_calculator' is not a valid callback",
                            ));
                        };

                        if input.len() < length {
                            Err(DecodeError::Incomplete(
                                "Input too short for callback length",
                            ))
                        } else {
                            let (slice_to_parse, remaining_after_slice) = input.split_at(length);
                            let (rem_in_slice, children) = parse_expansion_symbols(
                                slice_to_parse,
                                expansion,
                                grammar,
                                &mut temp_context,
                            )?;

                            if !rem_in_slice.is_empty() {
                                Err(DecodeError::Invalid("Expansion did not consume slice"))
                            } else {
                                Ok((remaining_after_slice, children))
                            }
                        }
                    } else {
                        parse_expansion_symbols(input, expansion, grammar, &mut temp_context)
                    };

                    if let Ok((remaining_input, children)) = parse_result {
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

fn parse_expansion_symbols<'a>(
    input: &'a [u8],
    expansion: &'a Expansion,
    grammar: &'a Grammar,
    context: &mut BTreeMap<String, Arc<DerivationTree>>,
) -> DecodeResult<'a, Vec<Arc<DerivationTree>>> {
    let mut remaining_input = input;
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
