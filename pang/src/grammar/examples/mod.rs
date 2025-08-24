//! Some example grammars.

use crate::{
    grammar,
    grammar::{Expansion, Grammar, exp},
    symbol::{nt, terminals::literal::t},
};

pub mod tlv;
pub mod xml;

/// ASCII letters
pub const ASCII_LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
/// ASCII digits
pub const DIGITS: &str = "0123456789";

/// Create a vector of expansions from given string.
pub fn srange(chars: &str) -> Vec<Expansion> {
    let mut expansions = Vec::new();
    for c in chars.chars() {
        // For each character in the string, create a terminal symbol.
        // We need to convert the char to a byte slice.
        // let mut buf = [0; 4]; // A char can be up to 4 bytes in UTF-8
        // let bytes = c.encode_utf8(&mut buf).as_bytes();

        // Create an expansion that is just this single terminal.
        let expansion = exp(vec![t(&c.to_string())]);

        // Add this expansion as one of the possible choices.
        expansions.push(expansion);
    }
    expansions
}

/// A simple expression grammar.
///
/// ```text
/// <start>   -> <expr>
/// <expr>    -> <term> "+" <expr> | <term> "-" <expr> | <term>
/// <term>    -> <factor> "*" <term> | <factor> "/" <term> | <factor>
/// <factor>  -> "+" <factor> | "-" <factor> | "(" <expr> ")"
///              | <integer> "." <integer> | <integer>
/// <integer> -> <digit> <integer> | <digit>
/// <digit>   -> "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
/// ```
pub fn expr_grammar() -> Grammar {
    grammar! {
        "start" => vec![
            exp(vec![nt("expr")])
        ],
        "expr" => vec![
            exp(vec![nt("term"), t("+"), nt("expr")]),
            exp(vec![nt("term"), t("-"), nt("expr")]),
            exp(vec![nt("term")]),
        ],
        "term" => vec![
            exp(vec![nt("factor"), t("*"), nt("term")]),
            exp(vec![nt("factor"), t("/"), nt("term")]),
            exp(vec![nt("factor")]),
        ],
        "factor" => vec![
            exp(vec![t("+"), nt("factor")]),
            exp(vec![t("-"), nt("factor")]),
            exp(vec![t("("), nt("expr"), t(")")]),
            exp(vec![nt("integer"), t("."), nt("integer")]),
            exp(vec![nt("integer")]),
        ],
        "integer" => vec![
            exp(vec![nt("digit"), nt("integer")]),
            exp(vec![nt("digit")])
        ],
        "digit" => srange(DIGITS)
    }
}

/// A simple grammar.
pub fn c_sample_grammar() -> Grammar {
    grammar! {
        "start" => vec![exp(vec![nt("A"), nt("B")])],
        "A" => vec![
            exp(vec![t("a"), nt("B"), t("c")]),
            exp(vec![nt("A"), nt("B")])
        ],
        "B" => vec![exp(vec![t("b"), nt("C")]), exp(vec![nt("D")])],
        "C" => vec![exp(vec![t("c")])],
        "D" => vec![exp(vec![t("d")])]
    }
}
