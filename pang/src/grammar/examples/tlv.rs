//! TLV [`Grammar`] Example

use crate::{
    grammar::{Grammar, exp, exp_with_opts},
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

use crate::grammar;

/// Generate a TLV grammar.
pub fn tlv_grammar() -> Grammar {
    grammar! {
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
            exp_with_opts(
                vec![t_dyn()],
                opts!("length_is" => "len")
            )
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
        "asn1-tlv-value" => vec![
            exp_with_opts(
                vec![t_dyn()],
                opts!("length_is" => "asn1-tlv-len")
            )
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use crate::grammar::{
        asn1_tlv_grammar,
        examples::tlv::{nest_tlv_grammar, tlv_grammar},
    };

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            let _ = env_logger::try_init();
        });
    }

    #[test]
    fn test_tlv_grammar() {
        setup_logger();
        let input = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08];
        let grammar = tlv_grammar();
        let tree = grammar.parse_combinator(input, "start").unwrap();
        assert_eq!(tree.to_bytes(), input);
    }

    #[test]
    fn test_nest_tlv_grammar() {
        setup_logger();
        let grammar = nest_tlv_grammar();
        let input = &[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08];
        let tree = grammar.parse_combinator(input, "start").unwrap();
        assert_eq!(tree.to_bytes(), input);

        let input = &[
            0x01, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x09, // length
            0x01, 0x00, 0x00, 0x00, // nest-type
            0x00, 0x00, 0x00, 0x01, // nest-length
            0x01, // nest-value
        ];
        let tree = grammar.parse_combinator(input, "start").unwrap();
        assert_eq!(tree.to_bytes(), input);

        let input = &[
            0x01, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x18, // length
            0x01, 0x00, 0x00, 0x00, // nest-type
            0x00, 0x00, 0x00, 0x10, // nest-length
            0x01, 0x00, 0x00, 0x00, // nest-nest-type
            0x00, 0x00, 0x00, 0x08, // nest-nest-length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // nest-nest-value
        ];
        let tree = grammar.parse_combinator(input, "start").unwrap();
        assert_eq!(tree.to_bytes(), input);
    }

    #[test]
    fn test_asn1_grammar() {
        setup_logger();
        let grammar = asn1_tlv_grammar();

        let input = &[0x02, 0x01, 0x00];
        let tree = grammar.parse_combinator(input, "asn1-tlv").unwrap();
        assert_eq!(tree.to_bytes(), input);

        let input = &[0x05, 0x00];
        let tree = grammar.parse_combinator(input, "asn1-tlv").unwrap();
        assert_eq!(tree.to_bytes(), input);
    }
}
