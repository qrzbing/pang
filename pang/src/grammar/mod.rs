//! Grammar can show the structure and syntax of language.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr,
    sync::Arc,
};

use log::error;

use crate::{
    DerivationTree,
    symbol::{DecodeError, Symbol},
};

pub mod macros;

/// Create a new Expansion with empty callbacks.
pub fn exp<T: Into<Vec<Symbol>>>(symbols: T) -> Expansion {
    Expansion {
        symbols: symbols.into(),
        decode_callback: None,
        encode_callback: None,
    }
}

/// Create a new Expansion with callbacks.
pub fn exp_cb<T: Into<Vec<Symbol>>>(
    symbols: T,
    decode_callback: DecodeCallback,
    encode_callback: EncodeCallback,
) -> Expansion {
    Expansion::with_callback(symbols.into(), Some(decode_callback), Some(encode_callback))
}

/// Create a new Expansion with decode callback.
pub fn exp_dc<T: Into<Vec<Symbol>>>(symbols: T, decode_callback: DecodeCallback) -> Expansion {
    Expansion::with_callback(symbols.into(), Some(decode_callback), None)
}

/// Create a new Expansion with decode callback.
pub fn exp_ec<T: Into<Vec<Symbol>>>(symbols: T, encode_callback: EncodeCallback) -> Expansion {
    Expansion::with_callback(symbols.into(), None, Some(encode_callback))
}

/// DecodeCallback takes an input slice and a context, returning the remaining
/// input and an owned slice to be parsed by the expansion.
/// The new owned slice allows for preprocessing, such as decompression.
pub type DecodeCallback = for<'a> fn(
    input: &'a [u8],
    context: &BTreeMap<String, Arc<DerivationTree>>,
) -> Result<(&'a [u8], Vec<u8>), DecodeError>;

/// EncodeCallback takes a node and returns a new node with user custom encoding.
pub type EncodeCallback = fn(node: Arc<DerivationTree>) -> Arc<DerivationTree>;

/// Expansion contains a sequence of symbols and options.
#[derive(Clone, Debug)]
pub struct Expansion {
    /// Symbols in the expansion.
    pub symbols: Vec<Symbol>,
    /// An optional callback to process input stream before parsing [`Symbol`]
    pub decode_callback: Option<DecodeCallback>,
    /// An optional callback to encode a node after generating an [`Expansion`]
    pub encode_callback: Option<EncodeCallback>,
}

impl PartialEq for Expansion {
    fn eq(&self, other: &Self) -> bool {
        self.symbols == other.symbols
            && match (self.decode_callback, other.decode_callback) {
                (None, None) => true,
                (Some(f), Some(g)) => ptr::fn_addr_eq(f, g),
                _ => false,
            }
            && match (self.encode_callback, other.encode_callback) {
                (None, None) => true,
                (Some(f), Some(g)) => ptr::fn_addr_eq(f, g),
                _ => false,
            }
    }
}

impl Eq for Expansion {}

impl Hash for Expansion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbols.hash(state);
        self.decode_callback.map(|cb| cb as usize).hash(state);
        self.encode_callback.map(|cb| cb as usize).hash(state);
    }
}

impl Expansion {
    /// Creates a new Expansion without callback.
    pub fn new(symbols: Vec<Symbol>) -> Self {
        Self {
            symbols,
            decode_callback: None,
            encode_callback: None,
        }
    }

    /// Creates a new Expansion with callback.
    pub fn with_callback(
        symbols: Vec<Symbol>,
        decode_callback: Option<DecodeCallback>,
        encode_callback: Option<EncodeCallback>,
    ) -> Self {
        Self {
            symbols,
            decode_callback: decode_callback,
            encode_callback: encode_callback,
        }
    }

    /// Get all nonterminals from a given expansion.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::{exp, nt, t};
    ///
    /// let expansion = exp([nt("expr"), t("+"), nt("term"), t("-"), nt("factor")]);
    /// let result = expansion.nonterminals();
    /// assert_eq!(result, ["expr", "term", "factor"]);
    /// ```
    pub fn nonterminals(&self) -> Vec<String> {
        self.symbols
            .iter()
            .filter_map(|symbol| match symbol {
                Symbol::NonTerminal { label }
                | Symbol::OneOrMore { label }
                | Symbol::ZeroOrMore { label } => Some(label.clone()),
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

                    // e.g., [nt("id"), t(b"="), nt("id")] -> "<id>=\"=\"<id>"
                    let expansion_str: String = expansion
                        .symbols
                        .iter()
                        .map(|symbol| symbol.to_string())
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
    /// use pang::{Grammar, language::expr_lang};
    /// let grammar1 = expr_lang().grammar;
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
    /// use pang::language::expr_lang;
    /// let grammar = expr_lang().grammar;
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
    /// use pang::language::expr_lang;
    /// let grammar = expr_lang().grammar;
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
    /// use pang::language::expr_lang;
    /// let grammar = expr_lang().grammar;
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
    /// use pang::{
    ///     grammar, exp, nt, t,
    ///     language::{expr_lang, xml_lang},
    /// };
    /// let grammar = expr_lang().grammar;
    /// assert_eq!(grammar.is_valid("start"), true);
    ///
    /// let grammar = grammar! {
    ///     "start" => [exp([nt("x")])],
    ///     "y" => [exp([t("1")])]
    /// };
    ///
    /// assert_eq!(grammar.is_valid("start"), false);
    ///
    /// let grammar = xml_lang().grammar;
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
}
