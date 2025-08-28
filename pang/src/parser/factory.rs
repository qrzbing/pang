//! src/parser/factory.rs

use crate::{
    grammar::Expansion,
    parser::expansion::{DefaultExpansionParser, ExpansionParser, LengthIsExpansionParser},
};

/// Select an appropriate expansion parser based on the given expansion.
pub fn get_expansion_parser(expansion: &Expansion) -> Box<dyn ExpansionParser> {
    // Check if the expansion has a "length_is" constraint.
    if expansion.options.contains_key("length_is") {
        return Box::new(LengthIsExpansionParser::default());
    }

    // TODO: Custom ExpansionParser
    // if let Some(parser_name) = expansion.options.get("parser").and_then(|v| v.as_str()) {
    //     match parser_name {
    //         "MyCustomParser" => return Box::new(MyCustomParser::new()),
    //         _ => {}
    //     }
    // }

    // Return DefaultExpansionParser by default.
    Box::new(DefaultExpansionParser::default())
}
