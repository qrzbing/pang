//! XML [`Grammar`] Example

use crate::grammar;
use crate::grammar::{
    Grammar,
    examples::{ASCII_LETTERS, DIGITS},
    exp, nt, srange, t,
};

/// Generate a XML grammar
pub fn xml_grammar() -> Grammar {
    let letter_chars = format!("{}{}{}{}{}", ASCII_LETTERS, DIGITS, "\"", "'", ".");
    let letter_space_chars = format!("{}{}{}{}{}{}", ASCII_LETTERS, DIGITS, "\"", "'", " ", "\t");
    grammar! {
        "start" => vec![exp(vec![nt("xml-tree")])],
        "xml-tree" => vec![
            exp(vec![nt("text")]),
            exp(vec![nt("xml-open-tag"), nt("xml-tree"), nt("xml-close-tag")]),
            exp(vec![nt("xml-openclose-tag")]),
            exp(vec![nt("xml-tree"), nt("xml-tree")]),
        ],
        "xml-open-tag" => vec![
            exp(vec![t(b"<"), nt("id"), t(b">")]),
            exp(vec![t(b"<"), nt("id"), t(b" "), nt("xml-attribute"), t(b">")]),
        ],
        "xml-openclose-tag" => vec![
            exp(vec![t(b"<"), nt("id"), t(b"/>")]),
            exp(vec![t(b"<"), nt("id"), t(b" "), nt("xml-attribute"), t(b"/>")]),
        ],
        "xml-close-tag" => vec![
            exp(vec![t(b"</"), nt("id"), t(b">")])
        ],
        "xml-attribute" => vec![
            exp(vec![nt("id"), t(b"="), nt("id")]),
            exp(vec![nt("xml-attribute"), t(b" "), nt("xml-attribute")]),
        ],
        "id" => vec![
            exp(vec![nt("letter")]),
            exp(vec![nt("id"), nt("letter")]),
        ],
        "text" => vec![
            exp(vec![nt("text"), nt("letter_space")]),
            exp(vec![nt("letter_space")]),
        ],
        "letter" => srange(&letter_chars),
        "letter_space" => srange(&letter_space_chars)
    }
}
