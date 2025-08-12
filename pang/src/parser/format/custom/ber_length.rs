use std::{collections::BTreeMap, sync::Arc};

use nom::{
    IResult,
    error::{ErrorKind, ParseError},
};

use crate::{
    grammar::{Expansion, t_dyn},
    parser::format::{CustomParser, SharedState, ber_to_usize, parse_ber_length_field},
    tree::{DerivationTree, new_node},
};

/// Parser for BER-encoded length field.
#[derive(Debug, Default)]
pub struct BerLengthParser {
    state: SharedState,
}

impl BerLengthParser {
    /// Create a new BerLengthParser.
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

impl CustomParser for BerLengthParser {
    fn parse<'a>(
        &self,
        input: &'a [u8],
        label: &str,
        _expansion: &Expansion,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> IResult<&'a [u8], Vec<Arc<DerivationTree>>> {
        let (remaining_input, consumed_slice) = parse_ber_length_field(input)?;
        log::debug!(
            "  |   |-- BerLengthParser consumed '{}' bytes for label '{}'",
            consumed_slice.len(),
            label
        );
        let length_value = ber_to_usize(consumed_slice).map_err(|_| {
            nom::Err::Failure(ParseError::from_error_kind(input, ErrorKind::Verify))
        })?;

        self.state.set(label.to_string(), length_value);
        let child_terminal = new_node(t_dyn(), Some(vec![]), Some(consumed_slice.to_vec()));
        Ok((remaining_input, vec![child_terminal]))
    }
}
