//! TLV Language Example

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::grammar;

use crate::tree::new_node;
use crate::{
    DerivationTree, Language, exp, exp_cb, nt, parser::callback::big_endian_bytes_to_usize,
    symbol::DecodeError, t_ber, t_bytes, t_bytes_val, t_dyn,
};

/// Generate a TLV language.
pub fn tlv_lang() -> Language {
    let grammar = grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![
            exp_cb(vec![nt("type"), nt("len"), nt("value")], None, Some(len_encode_callbackfn)),
        ],
        "type" => vec![exp(vec![t_bytes(4)])],
        "len" => vec![exp(vec![t_bytes(4)])],
        "value" => vec![
            exp_cb(vec![t_dyn()], Some(len_decode_callbackfn), None)
        ],
    };
    Language::new(grammar, "start", HashSet::new())
}

/// Generate a nested TLV grammar.
pub fn nest_tlv_lang() -> Language {
    let grammar = grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp_cb(vec![nt("type"), nt("len"), nt("value")], None, Some(len_encode_callbackfn)),],
        "type" => vec![exp(vec![t_bytes(4)])],
        "len" => vec![exp(vec![t_bytes(4)])],
        "value" => vec![
            exp(vec![nt("tlv")]),
            exp_cb(vec![t_dyn()], Some(len_decode_callbackfn), None)
        ],
    };
    Language::new(grammar, "start", HashSet::new())
}

/// An example Decode callback function for length.
pub fn len_decode_callbackfn<'a>(
    input: &'a [u8],
    context: &BTreeMap<String, Arc<DerivationTree>>,
) -> Result<(&'a [u8], Vec<u8>), DecodeError> {
    let len_tree = context.get("len").ok_or(DecodeError::Invalid(
        "Symbol not found in context for length calculation",
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
    let Some(children) = &node.children else {
        return node;
    };
    let value_tree = node.at(&[2]).expect("Value not found!");
    let real_len = value_tree.to_bytes().len();
    let len_bytes = (real_len as u32).to_be_bytes();
    // Generate a new Symbol
    let len_node_symbol = t_bytes_val(&len_bytes);

    let new_len_leaf_node = new_node(len_node_symbol, Some(vec![]));
    let old_len_symbol = children[1].symbol.clone();
    let new_len_subtree = new_node(old_len_symbol, Some(vec![new_len_leaf_node]));
    let mut new_children = children.clone();

    // Replace the old length subtree with the new one.
    new_children[1] = new_len_subtree;
    new_node(node.symbol.clone(), Some(new_children))
}

/// Generate an ASN.1 TLV grammar.
pub fn asn1_tlv_lang() -> Language {
    let grammar = grammar!(
        "asn1-tlv" => vec![
            exp_cb(
                vec![nt("asn1-tlv-type"),nt("asn1-tlv-len"),nt("asn1-tlv-value")],
                None, Some(len_encode_callbackfn)
            ),
        ],
        "asn1-tlv-type" => vec![
            exp(vec![t_bytes_val(&[0x02])]),  // Type: Integer
            exp(vec![t_bytes_val(&[0x04])]),  // Type: Octet String
            exp(vec![t_bytes_val(&[0x05])]),  // Type: Null
            exp(vec![t_bytes_val(&[0x06])]),  // Type: Object Identifier
            exp(vec![t_bytes_val(&[0x43])]),  // Type: Timeticks
        ],
        "asn1-tlv-len" => vec![
            exp(vec![t_ber()])
        ],
        "asn1-tlv-value" => vec![
            exp_cb(vec![t_dyn()], Some(asn1_tlv_len_decode_callbackfn), None)
        ],
    );
    Language::new(grammar, "asn1-tlv", HashSet::new())
}

fn asn1_tlv_len_decode_callbackfn<'a>(
    input: &'a [u8],
    context: &BTreeMap<String, Arc<DerivationTree>>,
) -> Result<(&'a [u8], Vec<u8>), DecodeError> {
    let len_tree = context.get("asn1-tlv-len").ok_or(DecodeError::Invalid(
        "Symbol not found in context for length calculation",
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
