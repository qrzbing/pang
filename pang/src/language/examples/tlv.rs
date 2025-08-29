//! TLV [`Grammar`] Example

use std::collections::HashSet;

use crate::grammar;
use crate::{
    grammar::{exp, exp_with_opts},
    language::Language,
    opts,
    symbol::{
        nt,
        terminals::{
            ber_length::t_ber,
            bytes::{t_bytes, t_bytes_val},
            dynamic::t_dyn,
        },
    },
};

/// Generate a TLV language.
pub fn tlv_lang() -> Language {
    let grammar = grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
        "type" => vec![exp(vec![t_bytes(4)])],
        "len" => vec![exp(vec![t_bytes(4)])],
        "value" => vec![
            exp_with_opts(
                vec![t_dyn()],
                opts!("length_is" => "len")
            )
        ],
    };
    Language::new(grammar, "start", HashSet::new())
}

/// Generate a nested TLV grammar.
pub fn nest_tlv_lang() -> Language {
    let grammar = grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
        "type" => vec![exp(vec![t_bytes(4)])],
        "len" => vec![exp(vec![t_bytes(4)])],
        "value" => vec![
            exp(vec![nt("tlv")]),
            exp_with_opts(
                vec![t_dyn()],
                opts!("length_is" => "len")
            )
        ],
    };
    Language::new(grammar, "start", HashSet::new())
}

/// Generate an ASN.1 TLV grammar.
pub fn asn1_tlv_lang() -> Language {
    let grammar = grammar!(
        "asn1-tlv" => vec![
            exp(vec![
                nt("asn1-tlv-type"),
                nt("asn1-tlv-len"),
                nt("asn1-tlv-value"),
            ])
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
            exp_with_opts(
                vec![t_dyn()],
                opts!("length_is" => "asn1-tlv-len")
            )
        ],
    );
    Language::new(grammar, "asn1-tlv", HashSet::new())
}
