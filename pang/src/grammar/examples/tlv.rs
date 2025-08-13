//! TLV [`Grammar`] Example

use crate::grammar::{Grammar, exp, exp_with_opts, nt, t_bytes, t_dyn};
use crate::{grammar, opts};

/// Generate a TLV grammar.
pub fn tlv_grammar() -> Grammar {
    grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
        "type" => vec![exp_with_opts(vec![t_bytes(4)], opts!("endian" => "little"))],
        "len" => vec![exp_with_opts(vec![t_bytes(4)], opts!("endian" => "little"))],
        "value" => vec![
            exp_with_opts(
                vec![t_dyn()],
                opts!("endian" => "little", "length_is" => "len")
            )
        ],
    }
}

/// Generate a nested TLV grammar.
pub fn nest_tlv_grammar() -> Grammar {
    grammar! {
        "start" => vec![exp(vec![nt("tlv")])],
        "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
        "type" => vec![exp_with_opts(vec![t_bytes(4)], opts!("endian" => "little"))],
        "len" => vec![exp_with_opts(vec![t_bytes(4)], opts!("endian" => "little"))],
        "value" => vec![
            exp(vec![nt("tlv")]),
            exp_with_opts(
                vec![t_dyn()],
                opts!("endian" => "little", "length_is" => "len")
            ),
        ],
    }
}
