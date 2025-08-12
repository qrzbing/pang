use super::*;

impl FormatParser {
    pub(super) fn parse_terminal<'a>(
        &self,
        input: &'a [u8],
        kind: &TerminalKind,
        symbol: &Symbol,
        _expansion: &Expansion,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> IResult<&'a [u8], Arc<DerivationTree>> {
        match kind {
            TerminalKind::Literal(val) => {
                let (i, m) = nom::bytes::complete::tag(val.as_slice())(input)?;
                Ok((i, new_node(symbol.clone(), Some(vec![]), Some(m.to_vec()))))
            }
            TerminalKind::Binary(bin_kind) => match bin_kind {
                BinaryKind::Bytes { size } => {
                    let (i, c) = nom::bytes::complete::take(*size)(input)?;
                    Ok((i, new_node(symbol.clone(), Some(vec![]), Some(c.to_vec()))))
                }
                BinaryKind::Bits { size } => {
                    let mut take_parser =
                        nom::bits::complete::take::<_, u64, _, nom::error::Error<(&[u8], usize)>>(
                            *size,
                        );
                    let (i, c) = nom::bits::bits(&mut take_parser)(input)?;
                    let value_bytes = c.to_be_bytes().to_vec();
                    Ok((i, new_node(symbol.clone(), Some(vec![]), Some(value_bytes))))
                }
                BinaryKind::Dynamic => {
                    log::error!(
                        "Unhandled Dynamic terminal. Did you forget to register a custom parser?"
                    );
                    Err(Err::Failure(ParseError::from_error_kind(
                        input,
                        ErrorKind::Fix,
                    )))
                }
            },
        }
    }
}
