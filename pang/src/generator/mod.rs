//! [`Generator`] can generate random derivation trees from a given [`Grammar`].

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::Arc,
};

use rand::{Rng, RngCore};

use crate::{
    grammar::{BinaryKind, Expansion, Grammar, Symbol, TerminalKind, t},
    tree::{DerivationTree, TreeFixer, new_node},
};

mod expansion;

/// CostStrategy defines the direction of expansion.
#[derive(Debug)]
pub enum CostStrategy {
    /// Expansion with the minimum cost.
    Min,
    /// Expansion with the maximum cost.
    Max,
}

/// ExpansionStrategy defines the strategy of expansion.
#[derive(Clone, Copy, Debug)]
pub enum ExpansionStrategy {
    /// Expansion randomly.
    Random,
    /// Expansion with the minimum cost.
    MinCost,
    /// Expansion with the maximum cost.
    MaxCost,
}

/// This helper function checks if an [`Expansion`] contains non-deterministic symbols
/// like Binary, which should not be cached.
fn is_expansion_cacheable(expansion: &Expansion) -> bool {
    for symbol in &expansion.symbols {
        if let Symbol::Terminal { kind } = symbol {
            if matches!(kind, TerminalKind::Binary(_)) {
                return false; // Found a Binary symbol, so this is not cacheable
            }
        }
    }
    true // No binary symbols found, it's cacheable
}

fn expansion_to_children(expansion: &Expansion) -> Vec<Arc<DerivationTree>> {
    if expansion.symbols.is_empty() {
        return vec![new_node(t(b""), Some(vec![]), None)];
    }
    expansion
        .symbols
        .iter()
        .map(|symbol| match symbol.clone() {
            s @ Symbol::NonTerminal { .. } => new_node(s, None, None),
            ref s @ Symbol::Terminal { ref kind } => match kind {
                TerminalKind::Literal(..) => new_node(s.clone(), Some(vec![]), None),
                TerminalKind::Binary(bin_kind) => {
                    let mut rng = rand::rng();
                    let value = match bin_kind {
                        BinaryKind::Bytes { size } => {
                            let mut bytes = vec![0u8; *size];
                            rng.fill_bytes(&mut bytes);
                            bytes
                        }
                        BinaryKind::Bits { size } => {
                            let byte_size = (*size + 7) / 8;
                            let mut vec = vec![0u8; byte_size];
                            rng.fill_bytes(&mut vec);
                            vec
                        }
                        BinaryKind::Dynamic => {
                            let size = rng.random_range(8..=16); // TODO: Dynamic size
                            let mut vec = vec![0u8; size];
                            rng.fill_bytes(&mut vec);
                            vec
                        }
                    };
                    new_node(s.clone(), Some(vec![]), Some(value))
                }
            },
        })
        .collect()
}

/// [`Generator`] is a random [`DerivationTree`] generator.
#[derive(Debug)]
pub struct Generator {
    /// The grammar used for generating trees.
    pub grammar: Grammar,
    /// The start symbol of the grammar.
    pub start_symbol: Symbol,
    /// The minimum number of nonterminals in the generated tree.
    pub min_nonterminals: u32,
    /// The maximum number of nonterminals in the generated tree.
    pub max_nonterminals: u32,

    // Cache
    expansion_cache: RefCell<HashMap<Expansion, Vec<Arc<DerivationTree>>>>,
    expansion_invocations: Cell<u64>,
    expansion_invocations_cached: Cell<u64>,

    // Expansion costs
    symbol_costs: HashMap<String, f64>,
    expansion_costs: HashMap<Expansion, f64>,

    fixers: Vec<Arc<dyn TreeFixer>>,
}

impl Generator {
    /// Create a new [`Generator`] with the given grammar and start symbol.
    pub fn new(
        grammar: Grammar,
        start_symbol: &str,
        min_nonterminals: u32,
        max_nonterminals: u32,
        fixers: Vec<Arc<dyn TreeFixer>>,
    ) -> Self {
        let symbol = Symbol::NonTerminal {
            label: start_symbol.to_string(),
        };
        let mut generator = Self {
            grammar,
            start_symbol: symbol,
            min_nonterminals,
            max_nonterminals,

            // Cache
            expansion_cache: RefCell::new(HashMap::new()),
            expansion_invocations: Cell::new(0),
            expansion_invocations_cached: Cell::new(0),

            // Expansion costs
            symbol_costs: HashMap::new(),
            expansion_costs: HashMap::new(),

            fixers,
        };
        generator.precompute_costs();

        generator
    }

    #[allow(dead_code)]
    fn expand_node(&self, node: &Arc<DerivationTree>) -> Arc<DerivationTree> {
        self.expand_node_randomly(node)
    }

    fn process_chosen_children(
        &self,
        chosen_children: Vec<Arc<DerivationTree>>,
        _expansion: &Expansion,
    ) -> Vec<Arc<DerivationTree>> {
        chosen_children
    }

    fn init_tree(&self) -> Arc<DerivationTree> {
        new_node(self.start_symbol.clone(), None, None)
    }

    /// Expand the tree starting from the root node.
    pub fn generate_tree(&self) -> Arc<DerivationTree> {
        let tree = self.init_tree();
        self.expand_tree(tree).fix_tree(&self.grammar, &self.fixers)
    }

    /// Generate a random [`DerivationTree`] and return a vector of bytes.
    pub fn generate(&self) -> Vec<u8> {
        self.generate_tree().all_terminals()
    }

    /// Print cache stats.
    pub fn print_cache_stats(&self) {
        let total = self.expansion_invocations.get();
        if total == 0 {
            println!("No expansions were performed.");
            return;
        }
        let cached = self.expansion_invocations_cached.get();
        let percentage = (cached as f64 * 100.0) / (total as f64);
        println!(
            "Cache Stats: {:.2}% of invocations were cached ({} / {}).",
            percentage, cached, total
        );
    }

    fn precompute_costs(&mut self) {
        let symbols: Vec<String> = self.grammar.keys().cloned().collect();

        // Use fixed-point iteration to compute the minimum cost of each symbol
        for symbol in &symbols {
            self.symbol_costs.insert(symbol.clone(), f64::INFINITY);
        }

        loop {
            let mut changed = false;

            for symbol in &symbols {
                let current_cost = *self.symbol_costs.get(symbol).unwrap();

                // Loop all expansions
                if let Some(expansions) = self.grammar.get(symbol) {
                    let min_expansion_cost = expansions
                        .iter()
                        .map(|exp| {
                            // Calculate the cost of a nonterminal
                            let non_terminals = exp.nonterminals();
                            let cost: f64 = non_terminals
                                .iter()
                                .map(|nt| *self.symbol_costs.get(nt).unwrap_or(&f64::INFINITY))
                                .sum();
                            1.0 + cost
                        })
                        .reduce(f64::min)
                        .unwrap_or(f64::INFINITY);

                    if min_expansion_cost < current_cost {
                        self.symbol_costs.insert(symbol.clone(), min_expansion_cost);
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        for expansions in self.grammar.values() {
            for expansion in expansions {
                let non_terminals = expansion.nonterminals();
                let cost = 1.0
                    + non_terminals
                        .iter()
                        .map(|nt| self.symbol_cost(nt))
                        .sum::<f64>();
                self.expansion_costs.insert(expansion.clone(), cost);
            }
        }
    }

    fn symbol_cost(&self, symbol: &str) -> f64 {
        *self.symbol_costs.get(symbol).unwrap_or(&f64::INFINITY)
    }

    fn expansion_cost(&self, expansion: &Expansion) -> f64 {
        *self
            .expansion_costs
            .get(expansion)
            .unwrap_or(&f64::INFINITY)
    }

    /// Generate a [`DerivationTree`] from a given [`Symbol`].
    pub fn generate_from_symbol(&self, symbol: &str) -> Arc<DerivationTree> {
        let start_node = new_node(
            Symbol::NonTerminal {
                label: symbol.to_string(),
            },
            None,
            None,
        );
        self.expand_tree(start_node)
            .fix_tree(&self.grammar, &self.fixers)
    }
}
