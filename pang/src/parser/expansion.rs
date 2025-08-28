//! src/parser/expansion.rs

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::{
    grammar::{Expansion, Grammar},
    symbol::{DecodeError, DecodeResult},
    tree::DerivationTree,
};

/// ExpansionParser is a trait that defines how to parse an expansion from a byte slice.
pub trait ExpansionParser: Debug {
    /// Parse an input with given expansion and grammar.
    fn parse<'a>(
        &self,
        input: &'a [u8],
        expansion: &'a Expansion,
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Vec<Arc<DerivationTree>>>;
}

/// DefaultExpansionParser is the default way to parse with Expansion.
#[derive(Debug, Default)]
pub struct DefaultExpansionParser;

impl ExpansionParser for DefaultExpansionParser {
    fn parse<'a>(
        &self,
        input: &'a [u8],
        expansion: &'a Expansion,
        grammar: &'a Grammar,
        parent_context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Vec<Arc<DerivationTree>>> {
        let mut remaining_input = input;
        let mut children = Vec::new();
        let mut local_context = parent_context.clone();

        for symbol in &expansion.symbols {
            match symbol.parse(remaining_input, grammar, &mut local_context) {
                Ok((next_input, child_node)) => {
                    remaining_input = next_input;
                    children.push(child_node);
                }
                Err(e) => return Err(e),
            }
        }
        Ok((remaining_input, children))
    }
}

/// LengthIsExpansionParser supports `length_is=<label>` option in Expansion.
#[derive(Debug, Default)]
pub struct LengthIsExpansionParser;

impl ExpansionParser for LengthIsExpansionParser {
    fn parse<'a>(
        &self,
        input: &'a [u8],
        expansion: &'a Expansion,
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Vec<Arc<DerivationTree>>> {
        let len_label = expansion
            .options
            .get("length_is")
            .and_then(|v| v.as_str())
            .ok_or(DecodeError::Invalid(
                "Missing or invalid 'length_is' option",
            ))?;

        let length_node = context.get(len_label).ok_or(DecodeError::Invalid(
            "Length field node not found in context",
        ))?;

        let length = length_node
            .first_terminal_kind()
            .ok_or(DecodeError::Invalid("No terminal found for length field"))?
            .as_has_length()
            .ok_or(DecodeError::Invalid(
                "Length terminal does not implement HasLength",
            ))?
            .as_length()
            .ok_or(DecodeError::Invalid(
                "Could not determine length from terminal",
            ))?;

        if input.len() < length {
            return Err(DecodeError::Incomplete(
                "Input too short for specified length",
            ));
        }
        let (slice_to_parse, remaining_after_slice) = input.split_at(length);

        let default_parser = DefaultExpansionParser::default();
        let (remaining_in_slice, children) =
            default_parser.parse(slice_to_parse, expansion, grammar, context)?;

        if !remaining_in_slice.is_empty() {
            return Err(DecodeError::Invalid(
                "Expansion did not consume the entire length-specified slice",
            ));
        }

        Ok((remaining_after_slice, children))
    }
}
