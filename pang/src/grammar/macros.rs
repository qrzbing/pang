//! Some useful macros for the grammar.

/// Create a `GrammarOptions`
///
/// Examples:
///
/// Define a TLV grammar with options. This grammar is defined in examples/tlv.rs.
///
/// ```
/// use pang::grammar;
/// use pang::{
///     grammar::{exp, exp_with_opts},
///     opts,
///     symbol::{
///         nt,
///         terminals::{bytes::t_bytes, dynamic::t_dyn},
///     },
/// };
///
/// let tlv_grammar = grammar! {
///     "start" => vec![exp(vec![nt("tlv")])],
///     "tlv" => vec![exp(vec![nt("type"), nt("len"), nt("value")])],
///     "type" => vec![exp(vec![t_bytes(4)])],
///     "len" => vec![exp(vec![t_bytes(4)])],
///     "value" => vec![
///         exp_with_opts(
///             vec![t_dyn()],
///             opts!("length_is" => "len")
///         )
///     ],
/// };
/// ```
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
/// use pang::{
///     grammar,
///     grammar::exp,
///     symbol::{nt, terminals::literal::t}
/// };
///
/// let grammar = grammar! {
///     "start" => vec![exp(vec![nt("digit"), t("+"), nt("digit")])],
///     "digit" => vec![exp(vec![t("0"), t("1"), t("2"), t("3"), t("4")])],
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
