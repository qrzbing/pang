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
