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
///
/// Example:
///
/// ```
/// use std::{collections::HashMap, sync::Arc};
/// 
/// use crate::{
///     grammar::asn1_tlv_grammar,
///     parser::{
///         FormatParser, Parser,
///         format::{BerLengthParser, ParserFactory, ber_to_usize},
///     },
/// };
/// 
/// let mut parsers_registry = HashMap::new();
/// 
/// let factory: ParserFactory = Arc::new(|state| Box::new(BerLengthParser::new(state)));
/// parsers_registry.insert("BerLengthParser".to_string(), factory);
/// 
/// let parser = FormatParser::new(asn1_tlv_grammar(), "asn1-tlv", &parsers_registry);
/// 
/// let packet = [0x05, 0x00];
/// let result = parser.parse_forest(&packet);
/// 
/// for tree in result.unwrap() {
///     assert_eq!(tree.to_bytes(), packet);
/// }
/// 
/// let tree = parser.parse_first(&packet).unwrap();
/// assert_eq!(tree.to_bytes(), packet);
/// 
/// let child = tree.children.as_ref().unwrap();
/// assert_eq!(child.len(), 3);
/// 
/// assert_eq!(child[0].symbol.label(), "asn1-tlv-type");
/// assert_eq!(child[1].symbol.label(), "asn1-tlv-len");
/// assert_eq!(child[2].symbol.label(), "asn1-tlv-value");
/// 
/// let asn1_tlv_len = child[1].to_bytes();
/// let asn1_tlv_value = child[2].to_bytes();
/// assert_eq!(ber_to_usize(&asn1_tlv_len).unwrap(), asn1_tlv_value.len());
/// ```
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
