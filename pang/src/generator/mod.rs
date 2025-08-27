//! [`Generator`] can generate random derivation trees from a given [`Grammar`].

use std::{collections::HashMap, sync::Arc};

use libafl_bolts::rands::StdRand;
use rand::Rng;

use crate::{
    grammar::{Expansion, Grammar},
    symbol::{Symbol, terminals::literal::t},
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

fn expansion_to_children(expansion: &Expansion) -> Vec<Arc<DerivationTree>> {
    if expansion.symbols.is_empty() {
        return vec![new_node(t(""), Some(vec![]))];
    }
    expansion
        .symbols
        .iter()
        .map(|symbol| match symbol.clone() {
            s @ Symbol::NonTerminal { .. } => new_node(s, None),
            ref _s @ Symbol::Terminal { ref kind } => {
                let mut rng = StdRand::with_seed(0);
                kind.generate(&mut rng)
            }
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

    // Expansion costs
    symbol_costs: HashMap<String, f64>,
    expansion_costs: HashMap<Expansion, f64>,

    _fixers: Vec<Arc<dyn TreeFixer>>,
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

            // Expansion costs
            symbol_costs: HashMap::new(),
            expansion_costs: HashMap::new(),

            _fixers: fixers,
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
        new_node(self.start_symbol.clone(), None)
    }

    /// Expand the tree starting from the root node.
    pub fn generate_tree(&self) -> Arc<DerivationTree> {
        let tree = self.init_tree();
        // self.expand_tree(tree).fix_tree(&self.grammar, &self.fixers)
        tree
    }

    /// Generate a random [`DerivationTree`] and return a vector of bytes.
    pub fn generate(&self) -> String {
        self.generate_tree().all_terminals()
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
        );
        self.expand_tree(start_node)
        // .fix_tree(&self.grammar, &self.fixers)
    }
}
