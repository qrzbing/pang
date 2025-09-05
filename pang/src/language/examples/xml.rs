//! XML Language Example

use std::collections::HashSet;

use crate::{
    grammar::exp,
    language::{ASCII_LETTERS, DIGITS, Language, srange},
    symbol::{nt, terminals::literal::t},
};

use crate::grammar;

/// Generate a XML language
pub fn xml_lang() -> Language {
    let letter_chars = format!("{}{}{}{}{}", ASCII_LETTERS, DIGITS, "\"", "'", ".");
    let letter_space_chars = format!("{}{}{}{}{}{}", ASCII_LETTERS, DIGITS, "\"", "'", " ", "\t");
    let grammar = grammar! {
        "start" => vec![exp(vec![nt("xml-tree")])],
        "xml-tree" => vec![
            exp(vec![nt("text")]),
            exp(vec![nt("xml-open-tag"), nt("xml-tree"), nt("xml-close-tag")]),
            exp(vec![nt("xml-openclose-tag")]),
            exp(vec![nt("xml-tree"), nt("xml-tree")]),
        ],
        "xml-open-tag" => vec![
            exp(vec![t("<"), nt("id"), t(">")]),
            exp(vec![t("<"), nt("id"), t(" "), nt("xml-attribute"), t(">")]),
        ],
        "xml-openclose-tag" => vec![
            exp(vec![t("<"), nt("id"), t("/>")]),
            exp(vec![t("<"), nt("id"), t(" "), nt("xml-attribute"), t("/>")]),
        ],
        "xml-close-tag" => vec![
            exp(vec![t("</"), nt("id"), t(">")])
        ],
        "xml-attribute" => vec![
            exp(vec![nt("id"), t("="), nt("id")]),
            exp(vec![nt("xml-attribute"), t(" "), nt("xml-attribute")]),
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
    };
    Language::new(&grammar, "start", HashSet::new())
}
