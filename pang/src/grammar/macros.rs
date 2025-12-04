//! Some useful macros for the grammar.

/// Create a `Grammar`
///
/// Examples:
///
/// ```
/// use pang::{grammar, exp, nt, t};
///
/// let grammar = grammar! {
///     "start" => [exp([nt("digit"), t("+"), nt("digit")])],
///     "digit" => [exp([t("0"), t("1"), t("2"), t("3"), t("4")])],
/// };
/// ```
#[macro_export]
macro_rules! grammar {
    {
        $(
            $name:expr => $body:expr
        ),*
        $(,)?
    } => {
        {
            let mut grammar = $crate::grammar::Grammar::new();
            $(
                grammar.insert($name.to_string(), $body.into());
            )*
            grammar
        }
    };
}

/// Registers a struct field into a `Grammar`.
/// 
/// # Examples
/// 
/// ```ignore
/// struct TestHeader {
///     magic_number: u8,
///     version: u8,
///     // ...
/// }
/// 
/// let mut rules = Grammar::new();
/// let mut seq = vec![];
/// let prefix = "TestHeader";
/// 
/// register_field!(rules, seq, prefix, "magic_number", u8);
/// register_field!(rules, seq, prefix, "version", u8);
/// ```
#[macro_export]
macro_rules! register_field {
    ($rules:ident, $seq:ident, $prefix:expr, $name:literal, $type:ty) => {
        let field_rule_name = format!("{}.{}", $prefix, $name);

        // Update sub-grammar recursively
        $rules = $rules.extend_grammar(&<$type as ToGrammar>::grammar());

        // Middleware rule: Header.field -> Type
        $rules.insert(
            field_rule_name.clone(),
            vec![exp(vec![nt(&<$type as PangLabel>::label())])],
        );

        // Add to grammar
        $seq.push(nt(&field_rule_name));
    };
}

#[cfg(test)]
mod tests {
    use crate::{Grammar, PangLabel, ToGrammar, exp, nt};

    #[test]
    fn test_register_field() {
        let mut rules = Grammar::new();
        let mut seq = vec![];
        let prefix = "TestHeader";

        // TestHeader.magic_number -> u8
        register_field!(rules, seq, prefix, "magic_number", u8);

        assert_eq!(seq.len(), 1, "Sequence should have 1 item");
        assert_eq!(
            seq[0],
            nt("TestHeader.magic_number"),
            "Sequence should verify intermediate rule"
        );

        assert!(
            rules.contains_key("TestHeader.magic_number"),
            "Intermediate rule missing"
        );

        let intermediate_rule_body = &rules["TestHeader.magic_number"];
        assert_eq!(
            intermediate_rule_body[0].symbols[0],
            nt("u8"),
            "Intermediate rule should point to field type label"
        );

        assert!(rules.contains_key("u8"), "Sub-grammar was not merged!");
    }
}
