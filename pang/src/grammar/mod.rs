//! Grammar can show the structure and syntax of language.

use std::collections::{BTreeMap, HashMap, HashSet};

use log::error;
use serde_json::Value as GrammarOptionValue;

pub mod examples;
pub use examples::{
    ASCII_LETTERS, DIGITS, c_sample_grammar, expr_grammar, srange, xml::xml_grammar,
};
pub mod macros;
pub mod symbol;
pub use symbol::{Symbol, TerminalKind, nt, t, t_bits, t_bytes, t_dyn};

/// Grammar can extend to do some user-defined actions by GrammarOptions.
pub type GrammarOptions = BTreeMap<String, GrammarOptionValue>;

/// Create a new Expansion with empty options.
pub fn exp(symbols: Vec<Symbol>) -> Expansion {
    Expansion {
        symbols,
        options: GrammarOptions::new(),
    }
}

/// Create a new Expansion with options.
pub fn exp_with_opts(symbols: Vec<Symbol>, options: GrammarOptions) -> Expansion {
    Expansion { symbols, options }
}

/// Expansion contains a sequence of symbols and options.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expansion {
    /// Symbols in the expansion.
    pub symbols: Vec<Symbol>,
    /// Options for the expansion.
    pub options: GrammarOptions,
}

/// Grammar contains a set of expansions.
pub type Grammar = HashMap<String, Vec<Expansion>>;

/// Get all nonterminals from a given expansion.
///
/// # Examples
///
/// ```
/// use pang::grammar::{is_nonterminal, t, nt, exp, nonterminals};
///
/// let expansion = exp(vec![nt("expr"), t(b"+"), nt("term"), t(b"-"), nt("factor")]);
/// let result = nonterminals(&expansion);
/// assert_eq!(result, vec!["expr", "term", "factor"]);
/// ```
pub fn nonterminals(expansion: &Expansion) -> Vec<String> {
    expansion
        .symbols
        .iter()
        .filter_map(|symbol| match symbol {
            Symbol::NonTerminal { label } => Some(label.clone()),
            _ => None,
        })
        .collect()
}

/// Checks if the given symbol is a nonterminal.
///
/// # Examples
///
/// ```
/// use pang::grammar::{is_nonterminal, t, nt};
///
/// let non_terminal = nt("expr");
/// let terminal = t(b"+");
///
/// assert!(is_nonterminal(&non_terminal));
/// assert!(!is_nonterminal(&terminal));
/// ```
pub fn is_nonterminal(symbol: &Symbol) -> bool {
    matches!(symbol, Symbol::NonTerminal { .. })
}

/// Extend a grammar with another one.
///
/// # Examples
///
/// ```
/// use pang::grammar::{extend_grammar, expr_grammar, Grammar};
///
/// let grammar1 = expr_grammar();
/// let grammar2 = Grammar::new();
///
/// let extend = extend_grammar(&grammar2, &grammar1);
/// assert_eq!(extend.len(), grammar1.len());
/// ```
pub fn extend_grammar(grammar: &Grammar, extension: &Grammar) -> Grammar {
    let mut new_grammar = grammar.clone();
    new_grammar.extend(extension.clone());
    new_grammar
}

/// Returns a tuple of two sets: defined nonterminals and used nonterminals
fn def_used_nonterminals(
    grammar: &Grammar,
    start_symbol: &str,
) -> (Option<HashSet<String>>, Option<HashSet<String>>) {
    let mut defined_nonterminals = HashSet::new();
    let mut used_nonterminals = HashSet::new();
    used_nonterminals.insert(start_symbol.to_string());

    for (label, expansions) in grammar {
        defined_nonterminals.insert(label.clone());
        if expansions.is_empty() {
            error!("Grammar entry '{}' has no expansions", label);
            return (None, None);
        }

        for expansion in expansions {
            used_nonterminals.extend(nonterminals(expansion));
        }
    }

    (Some(defined_nonterminals), Some(used_nonterminals))
}

/// Finds all nonterminals that can be reached from the start symbol.
fn reachable_nonterminals(grammar: &Grammar, start_symbol: &str) -> HashSet<String> {
    let mut reachable = HashSet::new();
    let mut to_visit = vec![start_symbol.to_string()];

    // The start symbol is always reachable.
    reachable.insert(start_symbol.to_string());

    // A depth-first search starts from the `start_symbol`.
    while let Some(symbol) = to_visit.pop() {
        // If the current symbol has rules in the grammar...
        if let Some(expansions) = grammar.get(&symbol) {
            // ...iterate through all its possible expansions.
            for expansion in expansions {
                // Find all nonterminals in the current expansion.
                for nonterminal in nonterminals(expansion) {
                    // Try to insert the nonterminal into the `reachable` set.
                    if reachable.insert(nonterminal.clone()) {
                        // If it's a newly discovered nonterminal,
                        // add it to `to_visit` list.
                        to_visit.push(nonterminal);
                    }
                }
            }
        }
    }

    reachable
}

/// Unreachable nonterminals are all_defined_nonterminals - reachable_nonterminals.
fn unreachable_nonterminals(grammar: &Grammar, start_symbol: &str) -> HashSet<String> {
    let all_defined_nonterminals: HashSet<String> = grammar.keys().cloned().collect();

    let reachable = reachable_nonterminals(grammar, start_symbol);
    all_defined_nonterminals
        .difference(&reachable)
        .cloned()
        .collect()
}

/// Checks if a grammar is valid.
///
/// # Examples
///
/// ```
/// use pang::grammar;
/// use pang::grammar::{is_valid_grammar, t, nt, exp, expr_grammar, xml_grammar};
/// let grammar = expr_grammar();
/// let is_valid = is_valid_grammar(&grammar, "start");
/// assert_eq!(is_valid, true);
///
/// let grammar = grammar! {
///     "start" => vec![exp(vec![nt("x")])],
///     "y" => vec![exp(vec![t(b"1")])]
/// };
///
/// let is_valid = is_valid_grammar(&grammar, "start");
/// assert_eq!(is_valid, false);
///
/// let grammar = xml_grammar();
/// // let display_grammar = DisplayGrammar::new(&grammar);
/// // println!("XML Grammar: {}", display_grammar);
/// assert_eq!(is_valid_grammar(&grammar, "start"), true);
/// ```
pub fn is_valid_grammar(grammar: &Grammar, start_symbol: &str) -> bool {
    let mut is_valid = true;

    let (defined_nonterminals, used_nonterminals) =
        match def_used_nonterminals(grammar, start_symbol) {
            (Some(d), Some(u)) => (d, u),
            _ => return false,
        };

    // defined_nonterminals - used_nonterminals
    for unused_nonterminal in defined_nonterminals.difference(&used_nonterminals) {
        error!(
            "{}: defined, but not used. Consider applying trim_grammar() on the grammar.",
            unused_nonterminal
        );
        is_valid = false;
    }

    // used_nonterminals - defined_nonterminals
    for undefined_nonterminal in used_nonterminals.difference(&defined_nonterminals) {
        error!("{}: used, but not defined.", undefined_nonterminal);
        is_valid = false;
    }

    let unreachable = unreachable_nonterminals(grammar, start_symbol);

    for unreachable_nonterminal in &unreachable {
        error!(
            "{}: unreachable from {}. Consider applying trim_grammar() on the grammar.",
            unreachable_nonterminal, start_symbol
        );
        is_valid = false;
    }

    is_valid
}

/// Trims a grammar by removing unused and unreachable nonterminals.
pub fn trim_grammar(grammar: &Grammar, start_symbol: &str) -> Grammar {
    let mut new_grammar = extend_grammar(grammar, &Grammar::new());

    let (defined_nonterminals, used_nonterminals) =
        match def_used_nonterminals(grammar, start_symbol) {
            (Some(d), Some(u)) => (d, u),
            _ => return new_grammar,
        };

    let unused: HashSet<_> = defined_nonterminals
        .difference(&used_nonterminals)
        .cloned()
        .collect();

    let unreachable = unreachable_nonterminals(grammar, start_symbol);

    for nonterminal_to_remove in unused.union(&unreachable) {
        new_grammar.remove(nonterminal_to_remove);
    }

    new_grammar
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_def_used_nonterminals() {
        let grammar = expr_grammar();
        let (defined_nonterminals, used_nonterminals) =
            match def_used_nonterminals(&grammar, "start") {
                (Some(d), Some(u)) => (d, u),
                _ => {
                    assert!(false);
                    return;
                }
            };
        assert_eq!(defined_nonterminals.len(), 6);
        assert_eq!(used_nonterminals.len(), 6);
    }

    #[test]
    fn test_reachable_nonterminals() {
        let grammar = expr_grammar();
        let reachable = reachable_nonterminals(&grammar, "start");
        assert_eq!(reachable.len(), 6);
    }

    #[test]
    fn test_unreachable_nonterminals() {
        let grammar = expr_grammar();
        let unreachable = unreachable_nonterminals(&grammar, "start");
        assert_eq!(unreachable.len(), 0);
    }
}
