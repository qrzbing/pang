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

impl Expansion {
    /// Get all nonterminals from a given expansion.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::{t, nt, exp};
    ///
    /// let expansion = exp(vec![nt("expr"), t(b"+"), nt("term"), t(b"-"), nt("factor")]);
    /// let result = expansion.nonterminals();
    /// assert_eq!(result, vec!["expr", "term", "factor"]);
    /// ```
    pub fn nonterminals(&self) -> Vec<String> {
        self.symbols
            .iter()
            .filter_map(|symbol| match symbol {
                Symbol::NonTerminal { label } => Some(label.clone()),
                _ => None,
            })
            .collect()
    }
}

/// Grammar contains a set of expansions.
pub type Grammar = HashMap<String, Vec<Expansion>>;

/// Extend Grammar with some useful methods.
pub trait GrammarExt {
    /// Extend a grammar with another one.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::{expr_grammar, Grammar, GrammarExt};
    ///
    /// let grammar1 = expr_grammar();
    /// let grammar2 = Grammar::new();
    ///
    /// let extend = grammar2.extend_grammar(&grammar1);
    /// assert_eq!(extend.len(), grammar1.len());
    /// ```
    fn extend_grammar(&self, extension: &Grammar) -> Grammar;

    /// Checks if a grammar is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar;
    /// use pang::grammar::{t, nt, exp, expr_grammar, xml_grammar, GrammarExt};
    /// let grammar = expr_grammar();
    /// assert_eq!(grammar.is_valid("start"), true);
    ///
    /// let grammar = grammar! {
    ///     "start" => vec![exp(vec![nt("x")])],
    ///     "y" => vec![exp(vec![t(b"1")])]
    /// };
    ///
    /// assert_eq!(grammar.is_valid("start"), false);
    ///
    /// let grammar = xml_grammar();
    /// // let display_grammar = DisplayGrammar::new(&grammar);
    /// // println!("XML Grammar: {}", display_grammar);
    /// assert_eq!(grammar.is_valid("start"), true);
    /// ```
    fn is_valid(&self, start_symbol: &str) -> bool;

    /// Returns a tuple of two sets: defined nonterminals and used nonterminals
    ///
    /// Examples
    ///
    /// ```
    /// use pang::grammar::{GrammarExt, expr_grammar};
    /// let grammar = expr_grammar();
    /// let (defined_nonterminals, used_nonterminals) = match grammar.def_used_nonterminals("start")
    /// {
    ///     (Some(d), Some(u)) => (d, u),
    ///     _ => {
    ///         assert!(false);
    ///         return;
    ///     }
    /// };
    /// assert_eq!(defined_nonterminals.len(), 6);
    /// assert_eq!(used_nonterminals.len(), 6);
    ///
    /// ```
    fn def_used_nonterminals(
        &self,
        start_symbol: &str,
    ) -> (Option<HashSet<String>>, Option<HashSet<String>>);

    /// Finds all nonterminals that can be reached from the start symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::{GrammarExt, expr_grammar};
    /// let grammar = expr_grammar();
    /// let reachable = grammar.reachable_nonterminals("start");
    /// assert_eq!(reachable.len(), 6);
    /// ```
    fn reachable_nonterminals(&self, start_symbol: &str) -> HashSet<String>;

    /// Unreachable nonterminals are all_defined_nonterminals - reachable_nonterminals.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::{GrammarExt, expr_grammar};
    /// let grammar = expr_grammar();
    /// let unreachable = grammar.unreachable_nonterminals("start");
    /// assert_eq!(unreachable.len(), 0);
    /// ```
    fn unreachable_nonterminals(&self, start_symbol: &str) -> HashSet<String>;

    /// Trims a grammar by removing unused and unreachable nonterminals.
    fn trim(&self, start_symbol: &str) -> Grammar;
}

impl GrammarExt for Grammar {
    fn extend_grammar(&self, extension: &Grammar) -> Grammar {
        let mut new_grammar = self.clone();
        new_grammar.extend(extension.clone());
        new_grammar
    }

    fn def_used_nonterminals(
        &self,
        start_symbol: &str,
    ) -> (Option<HashSet<String>>, Option<HashSet<String>>) {
        let mut defined_nonterminals = HashSet::new();
        let mut used_nonterminals = HashSet::new();
        used_nonterminals.insert(start_symbol.to_string());

        for (label, expansions) in self {
            defined_nonterminals.insert(label.clone());
            if expansions.is_empty() {
                error!("Grammar entry '{}' has no expansions", label);
                return (None, None);
            }

            for expansion in expansions {
                used_nonterminals.extend(expansion.nonterminals());
            }
        }

        (Some(defined_nonterminals), Some(used_nonterminals))
    }

    fn reachable_nonterminals(&self, start_symbol: &str) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut to_visit = vec![start_symbol.to_string()];

        // The start symbol is always reachable.
        reachable.insert(start_symbol.to_string());

        // A depth-first search starts from the `start_symbol`.
        while let Some(symbol) = to_visit.pop() {
            // If the current symbol has rules in the grammar...
            if let Some(expansions) = self.get(&symbol) {
                // ...iterate through all its possible expansions.
                for expansion in expansions {
                    // Find all nonterminals in the current expansion.
                    for nonterminal in expansion.nonterminals() {
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

    fn unreachable_nonterminals(&self, start_symbol: &str) -> HashSet<String> {
        let all_defined_nonterminals: HashSet<String> = self.keys().cloned().collect();

        let reachable = self.reachable_nonterminals(start_symbol);
        all_defined_nonterminals
            .difference(&reachable)
            .cloned()
            .collect()
    }

    fn is_valid(&self, start_symbol: &str) -> bool {
        let mut is_valid = true;

        let (defined_nonterminals, used_nonterminals) =
            match self.def_used_nonterminals(start_symbol) {
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

        let unreachable = self.unreachable_nonterminals(start_symbol);

        for unreachable_nonterminal in &unreachable {
            error!(
                "{}: unreachable from {}. Consider applying trim_grammar() on the grammar.",
                unreachable_nonterminal, start_symbol
            );
            is_valid = false;
        }

        is_valid
    }

    fn trim(&self, start_symbol: &str) -> Grammar {
        let mut new_grammar = self.extend_grammar(&Grammar::new());

        let (defined_nonterminals, used_nonterminals) =
            match self.def_used_nonterminals(start_symbol) {
                (Some(d), Some(u)) => (d, u),
                _ => return new_grammar,
            };

        let unused: HashSet<_> = defined_nonterminals
            .difference(&used_nonterminals)
            .cloned()
            .collect();

        let unreachable = self.unreachable_nonterminals(start_symbol);

        for nonterminal_to_remove in unused.union(&unreachable) {
            new_grammar.remove(nonterminal_to_remove);
        }

        new_grammar
    }
}
