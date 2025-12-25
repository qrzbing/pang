use nom::{IResult, number};
use pang::{PangLabel, ToGrammar, ToTree};

#[derive(PangLabel, ToGrammar, ToTree)]
pub struct BitsData {
    #[pang(bits = "0..4", label = "c")]
    pub a: u8,
    #[pang(bits = "4..8", label = "c")]
    pub b: u8,
}

impl BitsData {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, byte) = number::complete::u8(input)?;

        let a = byte & 0x0F;

        let b = (byte >> 4) & 0x0F;

        Ok((input, BitsData { a, b }))
    }
}

#[test]
fn test_combine_bits() {
    let grammar = BitsData::grammar();
    let bits_data_grammar = grammar.get("BitsData").expect("Can not find BitsData");
    assert_eq!(bits_data_grammar.len(), 1);

    let bits_data_grammar_first_label = &bits_data_grammar[0].symbols[0];
    assert_eq!(bits_data_grammar_first_label.label(), "BitsData.c");

    let bits_data = BitsData { a: 1, b: 2 };
    let tree = bits_data.to_tree();
    assert_eq!(tree.to_bytes(), &[0b0010_0001]);
}
#[test]
fn test_parse_bits_data() {
    let input = [0b0010_0001];

    let result = BitsData::parse(&input);

    assert!(result.is_ok());
    let (rest, data) = result.unwrap();

    assert!(rest.is_empty());

    assert_eq!(data.a, 1, "Parsed 'a' should be 1 (lower 4 bits)");
    assert_eq!(data.b, 2, "Parsed 'b' should be 2 (upper 4 bits)");

    assert_eq!(data.to_tree().to_bytes(), input);
}
