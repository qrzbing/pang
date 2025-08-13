//! Grammar can show the structure and syntax of language.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
    ops::{Deref, DerefMut},
};

use log::{debug, error};
use serde_json::Value as GrammarOptionValue;

// #[doc(hidden)]
pub mod examples;
pub use examples::{
    ASCII_LETTERS, DIGITS, c_sample_grammar, expr_grammar, srange, tlv::asn1_tlv_grammar,
    xml::xml_grammar,
};
pub mod macros;
pub mod symbol;
pub use symbol::{BinaryKind, Symbol, TerminalKind, nt, t, t_bits, t_bytes, t_dyn};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grammar(HashMap<String, Vec<Expansion>>);

impl Deref for Grammar {
    type Target = HashMap<String, Vec<Expansion>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Grammar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Grammar {
    type Item = (String, Vec<Expansion>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Vec<Expansion>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Grammar {
    type Item = (&'a String, &'a Vec<Expansion>);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Vec<Expansion>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_keys: Vec<_> = self.keys().collect();
        sorted_keys.sort();

        for (i, key) in sorted_keys.iter().enumerate() {
            writeln!(f, "<{}>", key)?;

            if let Some(expansions) = self.get(*key) {
                let num_expansions = expansions.len();

                for (j, expansion) in expansions.iter().enumerate() {
                    let prefix = if j == num_expansions - 1 {
                        "└── "
                    } else {
                        "├── "
                    };

                    // e.g., vec![nt("id"), t(b"="), nt("id")] -> "<id>=\"=\"<id>"
                    let expansion_str: String = expansion
                        .symbols
                        .iter()
                        .map(|symbol| symbol.display_symbol())
                        .collect();

                    writeln!(f, "{}{}", prefix, expansion_str)?;
                }
            }

            if i < sorted_keys.len() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Grammar {
    /// Create a new grammar
    pub fn new() -> Self {
        Grammar(HashMap::new())
    }

    /// Extend a grammar with another one.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::{expr_grammar, Grammar};
    ///
    /// let grammar1 = expr_grammar();
    /// let grammar2 = Grammar::new();
    ///
    /// let extend = grammar2.extend_grammar(&grammar1);
    /// assert_eq!(extend.len(), grammar1.len());
    /// ```
    pub fn extend_grammar(&self, extension: &Grammar) -> Grammar {
        let mut new_grammar = self.clone();
        new_grammar.extend(extension.clone());
        new_grammar
    }

    /// Returns a tuple of two sets: defined nonterminals and used nonterminals
    ///
    /// Examples
    ///
    /// ```
    /// use pang::grammar::expr_grammar;
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
    pub fn def_used_nonterminals(
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

    /// Finds all nonterminals that can be reached from the start symbol.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::expr_grammar;
    /// let grammar = expr_grammar();
    /// let reachable = grammar.reachable_nonterminals("start");
    /// assert_eq!(reachable.len(), 6);
    /// ```
    pub fn reachable_nonterminals(&self, start_symbol: &str) -> HashSet<String> {
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

    /// Unreachable nonterminals are all_defined_nonterminals - reachable_nonterminals.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::grammar::expr_grammar;
    /// let grammar = expr_grammar();
    /// let unreachable = grammar.unreachable_nonterminals("start");
    /// assert_eq!(unreachable.len(), 0);
    /// ```
    pub fn unreachable_nonterminals(&self, start_symbol: &str) -> HashSet<String> {
        let all_defined_nonterminals: HashSet<String> = self.keys().cloned().collect();

        let reachable = self.reachable_nonterminals(start_symbol);
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
    /// use pang::grammar::{t, nt, exp, expr_grammar, xml_grammar};
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
    pub fn is_valid(&self, start_symbol: &str) -> bool {
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

    /// Trims a grammar by removing unused and unreachable nonterminals.
    pub fn trim(&self, start_symbol: &str) -> Grammar {
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

    /// Computes all non-terminals that can derive an empty string (nullable).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use pang::grammar::{Grammar, exp, nt, t};
    /// use pang::grammar; // for grammar! macro
    ///
    /// let grammar = grammar! {
    ///     "S" => vec![exp(vec![nt("A"), nt("B")])],
    ///     "A" => vec![exp(vec![t(b"a")]), exp(vec![])], // A -> 'a' | ε
    ///     "B" => vec![exp(vec![nt("C"), nt("D")])],
    ///     "C" => vec![exp(vec![t(b"c")])],
    ///     "D" => vec![exp(vec![nt("A")])], // D -> A, and A is nullable
    /// };
    ///
    /// // Since A -> ε, A is nullable.
    /// // Since D -> A and A is nullable, D is also nullable.
    /// // B -> C D. Since C is not nullable, B is not nullable.
    /// // S -> A B. Since B is not nullable, S is not nullable.
    /// let nullable_set = grammar.compute_nullable();
    /// let expected: HashSet<String> = ["A".to_string(), "D".to_string()].into_iter().collect();
    /// assert_eq!(nullable_set, expected);
    /// ```
    pub fn compute_nullable(&self) -> HashSet<String> {
        let mut nullable = HashSet::new();
        loop {
            let before_len = nullable.len();
            for (non_terminal, expansions) in self.iter() {
                for expansion in expansions {
                    let all_symbols_are_nullable =
                        expansion.symbols.iter().all(|symbol| match symbol {
                            Symbol::NonTerminal { label } => nullable.contains(label),
                            Symbol::Terminal { kind } => match kind {
                                TerminalKind::Literal(value) => value.is_empty(),
                                _ => panic!("Do not use parser on Bytes / Bits"),
                            },
                        });
                    if all_symbols_are_nullable {
                        nullable.insert(non_terminal.clone());
                    }
                }
            }
            if nullable.len() == before_len {
                break;
            }
        }
        debug!("Nullable non-terminals: {:?}", nullable);
        nullable
    }
}
