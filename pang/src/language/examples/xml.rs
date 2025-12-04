//! XML Language Example

use std::collections::{HashMap, HashSet};

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
        "start" => [exp([nt("xml-tree")])],
        "xml-tree" => [
            exp([nt("text")]),
            exp([nt("xml-open-tag"), nt("xml-tree"), nt("xml-close-tag")]),
            exp([nt("xml-openclose-tag")]),
            exp([nt("xml-tree"), nt("xml-tree")]),
        ],
        "xml-open-tag" => [
            exp([t("<"), nt("id"), t(">")]),
            exp([t("<"), nt("id"), t(" "), nt("xml-attribute"), t(">")]),
        ],
        "xml-openclose-tag" => [
            exp([t("<"), nt("id"), t("/>")]),
            exp([t("<"), nt("id"), t(" "), nt("xml-attribute"), t("/>")]),
        ],
        "xml-close-tag" => [
            exp([t("</"), nt("id"), t(">")])
        ],
        "xml-attribute" => [
            exp([nt("id"), t("="), nt("id")]),
            exp([nt("xml-attribute"), t(" "), nt("xml-attribute")]),
        ],
        "id" => [
            exp([nt("letter")]),
            exp([nt("id"), nt("letter")]),
        ],
        "text" => [
            exp([nt("text"), nt("letter_space")]),
            exp([nt("letter_space")]),
        ],
        "letter" => srange(&letter_chars),
        "letter_space" => srange(&letter_space_chars)
    };
    Language::new(&grammar, "start", HashSet::new(), HashMap::new())
}
