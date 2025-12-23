use nom::{
    IResult,
    combinator::rest,
    number::complete::{be_u32, le_u32},
};
use pang::{PangLabel, ToGrammar, ToTree, U32be, U32le};

#[derive(PangLabel, ToGrammar, ToTree)]
pub struct ComplexData {
    #[pang(semantic = "U32be")]
    pub a: u32,
    #[pang(semantic = "U32le")]
    pub b: u32,
    pub data: Vec<u8>,
}

impl ComplexData {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, a) = be_u32(input)?;

        let (input, b) = le_u32(input)?;

        let (input, data_slice) = rest(input)?;

        let data = data_slice.to_vec();

        Ok((input, ComplexData { a, b, data }))
    }
}

#[test]
fn test_endian() {
    let grammar = ComplexData::grammar();
    // <ComplexData.a> -> <U32be>
    let a = grammar.get("ComplexData.a").unwrap();
    assert_eq!("U32be", a[0].symbols[0].label());

    let bytes = [
        0xde, 0xad, 0xbe, 0xef, 0xef, 0xbe, 0xad, 0xde, 0x12, 0x34, 0x56, 0x78,
    ];

    let (_, data) = ComplexData::parse(&bytes).expect("Can not parse msg!");

    assert_eq!(data.a, 0xdeadbeef);
    assert_eq!(data.b, 0xdeadbeef);

    let tree = data.to_tree();
    assert_eq!(tree.to_bytes(), bytes);
}
