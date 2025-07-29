//! Some useful macros for the grammar.

/// Create a `GrammarOptions`
///
/// Examples:
///
#[macro_export]
macro_rules! opts {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut options = $crate::grammar::GrammarOptions::new();
            $(
                options.insert($key.to_string(), serde_json::json!($value));
            )*
            options
        }
    };
}

/// Create a `Grammar`
///
/// Examples:
///
/// ```
/// use pang::grammar;
/// use pang::grammar::{exp, nt, t};
///
/// let grammar = grammar! {
///     "start" => vec![exp(vec![nt("digit"), t(b"+"), nt("digit")])],
///     "digit" => vec![exp(vec![t(b"0"), t(b"1"), t(b"2"), t(b"3"), t(b"4")])],
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
                grammar.insert($name.to_string(), $body);
            )*
            grammar
        }
    };
}
