//! TLV Language Example

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::{exp_dc, exp_ec, grammar};

use crate::{
    DerivationTree, Language, exp, new_node, nt, parser::callback::big_endian_bytes_to_usize,
    symbol::DecodeError, t_bytes_val, t_dyn, tl_ber, tl_bytes, tl_bytes_val,
};

/// Generate a TLV language.
pub fn tlv_lang() -> Language {
    let grammar = grammar! {
        "start" => [exp([nt("tlv")])],
        "tlv" => [
            exp_ec([tl_bytes("type", 4), tl_bytes("len", 4), nt("value")], len_encode_callbackfn),
        ],
        "value" => [
            exp_dc([t_dyn()], len_decode_callbackfn)
        ],
    };
    Language::new(&grammar, "start", HashSet::new())
}

/// Generate a nested TLV grammar.
pub fn nest_tlv_lang() -> Language {
    let grammar = grammar! {
        "start" => [exp([nt("tlv")])],
        "tlv" => [exp_ec([tl_bytes("type", 4), tl_bytes("len", 4), nt("value")], len_encode_callbackfn),],
        "value" => [
            exp([nt("tlv")]),
            exp_dc([t_dyn()], len_decode_callbackfn)
        ],
    };
    Language::new(&grammar, "start", HashSet::new())
}

/// An example Decode callback function for length.
pub fn len_decode_callbackfn<'a>(
    input: &'a [u8],
    context: &BTreeMap<String, Arc<DerivationTree>>,
) -> Result<(&'a [u8], Vec<u8>), DecodeError> {
    let len_tree = context.get("len").ok_or(DecodeError::Invalid(
        "Symbol not found in context for length calculation".into(),
    ))?;
    let len = big_endian_bytes_to_usize(&len_tree.to_bytes())?;

    // Check if the input has enough data.
    if input.len() < len {
        return Err(DecodeError::Incomplete(
            "Input is shorter than the length specified in the context",
        ));
    }

    let (slice_to_parse, remaining_input) = input.split_at(len);

    Ok((remaining_input, slice_to_parse.to_vec()))
}

/// An example Encode callback function for length.
/// TODO: Performance optimization
pub fn len_encode_callbackfn(node: Arc<DerivationTree>) -> Arc<DerivationTree> {
    let value_tree = node.at(&[2]).expect("Value not found!");
    let real_len = value_tree.to_bytes().len();
    let len_bytes = (real_len as u32).to_be_bytes();
    // Generate a new Symbol
    let len_node_symbol = tl_bytes_val("len", &len_bytes);
    let node_res = node.replace_by_path(&[1], new_node(len_node_symbol, Some(vec![])));

    node_res.expect("Failed to replace length node!")
}

/// Generate an ASN.1 TLV grammar.
pub fn asn1_tlv_lang() -> Language {
    let grammar = grammar!(
        "asn1-tlv" => [
            exp_ec(
                [nt("asn1-tlv-type"), tl_ber("asn1-tlv-len"), nt("asn1-tlv-value")],
                len_encode_callbackfn
            ),
        ],
        "asn1-tlv-type" => [
            exp([t_bytes_val(&[0x02])]),  // Type: Integer
            exp([t_bytes_val(&[0x04])]),  // Type: Octet String
            exp([t_bytes_val(&[0x05])]),  // Type: Null
            exp([t_bytes_val(&[0x06])]),  // Type: Object Identifier
            exp([t_bytes_val(&[0x43])]),  // Type: Timeticks
        ],
        "asn1-tlv-value" => [
            exp_dc([t_dyn()], asn1_tlv_len_decode_callbackfn)
        ],
    );
    Language::new(&grammar, "asn1-tlv", HashSet::new())
}

fn asn1_tlv_len_decode_callbackfn<'a>(
    input: &'a [u8],
    context: &BTreeMap<String, Arc<DerivationTree>>,
) -> Result<(&'a [u8], Vec<u8>), DecodeError> {
    let len_tree = context.get("asn1-tlv-len").ok_or(DecodeError::Invalid(
        "Symbol not found in context for length calculation".into(),
    ))?;
    let len = big_endian_bytes_to_usize(&len_tree.to_bytes())?;

    // Check if the input has enough data.
    if input.len() < len {
        return Err(DecodeError::Incomplete(
            "Input is shorter than the length specified in the context",
        ));
    }

    let (slice_to_parse, remaining_input) = input.split_at(len);

    Ok((remaining_input, slice_to_parse.to_vec()))
}
