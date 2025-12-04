//! Some example languages.

use std::collections::{HashMap, HashSet};

use crate::{
    grammar::{Expansion, exp},
    language::Language,
    symbol::{nt, terminals::literal::t},
};

use crate::grammar;

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
        let expansion = exp([t(&c.to_string())]);

        // Add this expansion as one of the possible choices.
        expansions.push(expansion);
    }
    expansions
}

/// A simple expression language.
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
pub fn expr_lang() -> Language {
    let grammar = grammar! {
        "start" => [
            exp([nt("expr")])
        ],
        "expr" => [
            exp([nt("term"), t("+"), nt("expr")]),
            exp([nt("term"), t("-"), nt("expr")]),
            exp([nt("term")]),
        ],
        "term" => [
            exp([nt("factor"), t("*"), nt("term")]),
            exp([nt("factor"), t("/"), nt("term")]),
            exp([nt("factor")]),
        ],
        "factor" => [
            exp([t("+"), nt("factor")]),
            exp([t("-"), nt("factor")]),
            exp([t("("), nt("expr"), t(")")]),
            exp([nt("integer"), t("."), nt("integer")]),
            exp([nt("integer")]),
        ],
        "integer" => [
            exp([nt("digit"), nt("integer")]),
            exp([nt("digit")])
        ],
        "digit" => srange(DIGITS)
    };

    Language::new(&grammar, "start", HashSet::new(), HashMap::new())
}

/// A simple language.
pub fn c_sample_lang() -> Language {
    let grammar = grammar! {
        "start" => [exp([nt("A"), nt("B")])],
        "A" => [
            exp([t("a"), nt("B"), t("c")]),
            exp([nt("A"), nt("B")])
        ],
        "B" => [exp([t("b"), nt("C")]), exp([nt("D")])],
        "C" => [exp([t("c")])],
        "D" => [exp([t("d")])]
    };

    Language::new(&grammar, "start", HashSet::new(), HashMap::new())
}
