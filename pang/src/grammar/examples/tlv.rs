//! TLV [`Grammar`] Example

use crate::{
    grammar,
    grammar::{Grammar, exp},
    symbol::{
        nt,
        terminals::{
            ber_length::t_ber,
            bytes::{t_bytes, t_bytes_val},
            length_is::t_length_is,
        },
    },
};

/// Generate a TLV grammar.
pub fn tlv_grammar() -> Grammar {
    grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
        "type" => vec![exp(vec![t_bytes(4)])],
        "len" => vec![exp(vec![t_bytes(4)])],
        "value" => vec![
            exp(
                vec![t_length_is("len")],
            )
        ],
    }
}

/// Generate a nested TLV grammar.
pub fn nest_tlv_grammar() -> Grammar {
    grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
        "type" => vec![exp(vec![t_bytes(4)])],
        "len" => vec![exp(vec![t_bytes(4)])],
        "value" => vec![
            exp(vec![nt("tlv")]),
            exp(
                vec![t_length_is("len")],
            ),
        ],
    }
}

/// Generate an ASN.1 TLV grammar.
pub fn asn1_tlv_grammar() -> Grammar {
    grammar!(
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
        "asn1-tlv-value" => vec![exp(vec![t_length_is("len")])],
    )
}
