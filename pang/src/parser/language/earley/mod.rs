//! This module implements Earley Parser algorithm, most of the code is
//! python-to-rust from [fuzzingbook](https://www.fuzzingbook.org/html/Parser.html#The-Earley-Parser).
//!
//! The Earley parser is a general parser that is able to parse any arbitrary CFG.
//! It was invented by Jay Earley [[Earley et al, 1970](https://doi.org/10.1145/362007.362035)] for use in computational
//! linguistics. While its computational complexity is $O(n^3)$,
//! for parsing strings with arbitrary grammars, it can parse strings with
//! unambiguous grammars in $O(n^2)$ time, and all [LR(k)](https://en.wikipedia.org/wiki/LR_parser) grammars in linear time
//! ($O(n)$ [[Joop M.I.M. Leo, 1991](https://doi.org/10.1016/0304-3975(91)90180-A)]). Further improvements – notably handling
//! epsilon rules – were invented by Aycock et al. [[John Aycock et al, 2002](https://ieeexplore.ieee.org/abstract/document/8139393/)].

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use log::debug;

use crate::{
    grammar::{Expansion, Grammar, Symbol, TerminalKind, exp, nt, t},
    parser::{
        Region,
        language::{LanguageParser, PHONY_START_SYMBOL, Parser},
    },
    tree::{DerivationTree, new_node},
};

#[cfg(test)]
mod earley_tests;

#[derive(Debug, Clone)]
struct State {
    name: String,
    expr: Vec<Symbol>,
    dot: usize,
    s_col_idx: usize,
    e_col_idx: usize,
}

impl State {
    fn new(name: &str, expr: &Expansion, dot: usize, s_col_idx: usize) -> Self {
        Self {
            name: name.to_string(),
            expr: expr.symbols.to_vec(),
            dot,
            s_col_idx,
            e_col_idx: s_col_idx,
        }
    }

    /// Check if the `dot` has moved beyond the last element in `expr`.
    fn is_finished(&self) -> bool {
        self.dot >= self.expr.len()
    }

    /// Returns the current symbol being parsed.
    fn at_dot(&self) -> Option<&Symbol> {
        self.expr.get(self.dot)
    }

    /// Generate a new [`State`] with the `dot` advanced one token.
    /// This represents an advance of the parsing.
    fn advance(&self) -> Self {
        let mut new_state = self.clone();
        new_state.dot += 1;
        new_state
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.expr.hash(state);
        self.dot.hash(state);
        self.s_col_idx.hash(state);
        self.e_col_idx.hash(state);
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.expr == other.expr
            && self.dot == other.dot
            && self.s_col_idx == other.s_col_idx
            && self.e_col_idx == other.e_col_idx
    }
}

impl Eq for State {}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut expansion_parts = Vec::new();
        for (i, symbol) in self.expr.iter().enumerate() {
            if i == self.dot {
                expansion_parts.push("|".to_string());
            }
            expansion_parts.push(symbol.display_symbol());
        }
        if self.dot == self.expr.len() {
            expansion_parts.push("|".to_string());
        }

        write!(
            f,
            "{: <12} := {: <20} ({}, {})",
            self.name,
            expansion_parts.join(" "),
            self.s_col_idx,
            self.e_col_idx
        )
    }
}

#[derive(Debug, Clone)]
struct Column {
    index: usize,
    token: Option<u8>,
    states: Vec<State>,
    unique_states: HashSet<State>,
}

impl Column {
    fn new(index: usize, token: Option<u8>) -> Self {
        Self {
            index,
            token,
            states: Vec::new(),
            unique_states: HashSet::new(),
        }
    }

    fn add(&mut self, state: State) -> bool {
        if self.unique_states.contains(&state) {
            false
        } else {
            self.unique_states.insert(state.clone());
            self.states.push(state);
            true
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let token_display = self
            .token
            .map(|t| (t as char).to_string())
            .unwrap_or_else(|| "(init)".to_string());

        writeln!(f, "{} chart[{}]", token_display, self.index)?;

        let finished_states: Vec<String> = self
            .states
            .iter()
            .filter(|state| state.is_finished())
            .map(|state| state.to_string())
            .collect();

        write!(f, "{}", finished_states.join("\n"))?;

        Ok(())
    }
}

fn multi_cartesian_product<T: Clone>(lists: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![vec![]];
    for list in lists {
        if list.is_empty() {
            return vec![];
        }
        result = result
            .into_iter()
            .flat_map(|existing_product| {
                list.iter().map(move |item| {
                    let mut new_product = existing_product.clone();
                    new_product.push(item.clone());
                    new_product
                })
            })
            .collect();
    }
    result
}

/// Earley Parser
#[derive(Debug)]
pub struct EarleyParser {
    grammar: Grammar,
    start_symbol: String,
    tokens: HashSet<String>,
    coalesce_tokens: bool,
    /// Include all non-terminals that can derive an empty string
    nullable: HashSet<String>,
}

impl EarleyParser {
    /// Create a new Earley Parser
    pub fn new(
        mut grammar: Grammar,
        start_symbol: &str,
        tokens: HashSet<String>,
        coalesce_tokens: bool,
    ) -> Self {
        let mut actual_start_symbol = start_symbol.to_string();

        let needs_phony_start = grammar.get(start_symbol).map_or(true, |r| r.len() != 1);

        if needs_phony_start {
            actual_start_symbol = PHONY_START_SYMBOL.to_string();
            grammar.insert(
                actual_start_symbol.clone(),
                vec![exp(vec![nt(start_symbol)])],
            );
        }

        let nullable = grammar.compute_nullable();

        Self {
            grammar,
            start_symbol: actual_start_symbol,
            tokens,
            coalesce_tokens,
            nullable,
        }
    }

    fn chart_parse(&self, text: &[u8]) -> Vec<Column> {
        let mut chart: Vec<Column> = (0..=text.len())
            .map(|i| {
                let token = if i == 0 {
                    None
                } else {
                    text.get(i - 1).copied()
                };
                Column::new(i, token)
            })
            .collect();

        let start_rule = &self.grammar[&self.start_symbol][0];
        let start_state = State::new(&self.start_symbol, start_rule, 0, 0);
        chart[0].add(start_state);

        self.fill_chart(&mut chart, text);
        chart
    }

    fn fill_chart(&self, chart: &mut Vec<Column>, text: &[u8]) {
        for i in 0..chart.len() {
            let mut state_idx = 0;
            while state_idx < chart[i].states.len() {
                let state = chart[i].states[state_idx].clone();

                if state.is_finished() {
                    self.complete(chart, i, &state);
                } else {
                    let symbol = state.at_dot().unwrap();
                    if symbol.is_nonterminal() {
                        self.predict(chart, i, symbol.label());

                        if self.nullable.contains(symbol.label()) {
                            let mut advanced_state = state.advance();
                            advanced_state.e_col_idx = i;
                            chart[i].add(advanced_state);
                        }
                    } else {
                        self.scan(chart, i, &state, symbol.value(), text);
                    }
                }
                state_idx += 1;
            }
        }
    }

    fn predict(&self, chart: &mut Vec<Column>, col_idx: usize, non_terminal: &str) {
        debug!("PREDICT for {} at column {}", non_terminal, col_idx);
        if let Some(expansions) = self.grammar.get(non_terminal) {
            for expansion in expansions {
                let new_state = State::new(non_terminal, expansion, 0, col_idx);
                chart[col_idx].add(new_state);
            }
        }
    }

    fn scan(
        &self,
        chart: &mut Vec<Column>,
        col_idx: usize,
        state: &State,
        token: &[u8],
        text: &[u8],
    ) {
        if col_idx < text.len() && text[col_idx..].starts_with(token) {
            debug!("SCAN success for token '{:?}' at column {}", token, col_idx);
            let new_pos = col_idx + token.len();
            let mut new_state = state.advance();
            new_state.e_col_idx = new_pos;
            chart[new_pos].add(new_state);
        }
    }

    fn complete(&self, chart: &mut Vec<Column>, col_idx: usize, finished_state: &State) {
        debug!(
            "COMPLETE for state '{}' spanning ({}, {})",
            finished_state.name, finished_state.s_col_idx, col_idx
        );
        let start_col_states = chart[finished_state.s_col_idx].states.clone();
        for state_in_start_col in &start_col_states {
            if let Some(symbol_at_dot) = state_in_start_col.at_dot() {
                if symbol_at_dot.is_nonterminal() && symbol_at_dot.label() == finished_state.name {
                    let mut new_state = state_in_start_col.advance();
                    new_state.e_col_idx = col_idx;
                    chart[col_idx].add(new_state);
                }
            }
        }
    }

    fn parse_prefix_first(&self, text: &[u8]) -> (usize, Option<Arc<DerivationTree>>) {
        let chart = self.chart_parse(text);

        if let Some(last_column) = chart.last() {
            for state in last_column.states.iter().rev() {
                if state.name == self.start_symbol && state.is_finished() && state.s_col_idx == 0 {
                    let mut memo = HashMap::new();
                    if let Some(tree) = self.extract_first_tree(&chart, state, &mut memo) {
                        return (text.len(), Some(tree));
                    }
                }
            }
        }

        (0, None)
    }

    fn extract_first_tree(
        &self,
        chart: &[Column],
        state: &State,
        memo: &mut HashMap<State, Option<Arc<DerivationTree>>>,
    ) -> Option<Arc<DerivationTree>> {
        // 1. Check memoization cache first. This caches both successes (Some) and failures (None).
        if let Some(cached) = memo.get(state) {
            return cached.clone();
        }

        let paths = self.find_paths(chart, state);

        // 2. Iterate through each possible derivation path (i.e., production rule).
        for path in paths {
            let mut children_for_this_path: Vec<Arc<DerivationTree>> = Vec::new();
            let mut path_is_viable = true;

            // 3. For the current path, try to find ONE valid subtree for each child.
            for (child_symbol, kind, child_state_opt) in path {
                match kind {
                    't' => {
                        // Terminals are always a success.
                        children_for_this_path.push(new_node(child_symbol, Some(vec![]), None));
                    }
                    'n' => {
                        // For non-terminals, recurse.
                        let child_state = child_state_opt.unwrap();
                        // We only need ONE child tree. If we find one, we continue.
                        if let Some(child_tree) = self.extract_first_tree(chart, &child_state, memo)
                        {
                            children_for_this_path.push(child_tree);
                        } else {
                            // If any child cannot form a tree, this entire path is invalid.
                            // Break the inner loop and try the next path.
                            path_is_viable = false;
                            break;
                        }
                    }
                    _ => unreachable!(),
                }
            }

            // 4. If we successfully found a child tree for every child in this path...
            if path_is_viable {
                // ...we have found our first complete tree!
                let tree = new_node(nt(&state.name), Some(children_for_this_path), None);

                // Cache the success and return immediately.
                memo.insert(state.clone(), Some(tree.clone()));
                return Some(tree);
            }
            // Otherwise, the loop continues to the next path.
        }

        // 5. If we've exhausted all paths and none produced a tree, cache the failure.
        memo.insert(state.clone(), None);
        None
    }

    fn extract_trees(
        &self,
        chart: &[Column],
        state: &State,
        memo: &mut HashMap<State, Vec<Arc<DerivationTree>>>,
    ) -> Vec<Arc<DerivationTree>> {
        if let Some(cached) = memo.get(state) {
            return cached.clone();
        }

        let mut all_derivations = Vec::new();
        let paths = self.find_paths(chart, state);

        for path in paths {
            let child_tree_options: Vec<Vec<Arc<DerivationTree>>> = path
                .into_iter()
                .map(|(child_symbol, kind, child_state_opt)| {
                    match kind {
                        't' => {
                            // Terminal
                            vec![new_node(child_symbol, Some(vec![]), None)]
                        }
                        'n' => {
                            // NonTerminal
                            let child_state = child_state_opt.unwrap();
                            self.extract_trees(chart, &child_state, memo)
                        }
                        _ => unreachable!(),
                    }
                })
                .collect();

            let combined_trees = multi_cartesian_product(child_tree_options);
            for children in combined_trees {
                let tree = new_node(nt(&state.name), Some(children), None);
                all_derivations.push(tree);
            }
        }

        memo.insert(state.clone(), all_derivations.clone());
        all_derivations
    }

    fn find_paths(
        &self,
        chart: &[Column],
        state: &State,
    ) -> Vec<Vec<(Symbol, char, Option<State>)>> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Initial status: (to process expr, end column index, current find path)
        queue.push_back((state.expr.clone(), state.e_col_idx, Vec::new()));

        while let Some((mut current_expr, k, path)) = queue.pop_front() {
            if let Some(last_symbol) = current_expr.pop() {
                // Process every symbol in expr from end to start
                match last_symbol {
                    Symbol::Terminal { kind } => match kind {
                        TerminalKind::Literal(value) => {
                            let new_k = k.saturating_sub(value.len());
                            let mut new_path = path.clone();
                            new_path.insert(0, (t(&value), 't', None));
                            queue.push_back((current_expr, new_k, new_path));
                        }
                        _ => panic!("Do not use parser on Bytes / Bits"),
                    },
                    Symbol::NonTerminal { ref label } => {
                        // For non-terminal, find all matching, completed status in col k.
                        for prev_state in &chart[k].states {
                            if prev_state.is_finished() && &prev_state.name == label {
                                let mut new_path = path.clone();
                                new_path.insert(0, (nt(label), 'n', Some(prev_state.clone())));
                                // Push last status(child status start index) to queue and continue back-tracing.
                                queue.push_back((
                                    current_expr.clone(),
                                    prev_state.s_col_idx,
                                    new_path,
                                ));
                            }
                        }
                    }
                }
            } else if k == state.s_col_idx {
                result.push(path);
            }
        }
        result
    }
}

impl Parser for EarleyParser {
    fn parse_first(&self, text: &[u8]) -> Result<Arc<DerivationTree>, String> {
        let (cursor, maybe_tree) = self.parse_prefix_first(text);

        if cursor < text.len() {
            return Err(format!("Syntax error at position {}", cursor));
        }

        match maybe_tree {
            Some(tree) => Ok(self.prune_tree(tree)),
            None => Err("Failed to parse a complete tree".to_string()),
        }
    }

    /// Parses the given text using the PEG parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use pang::{
    ///     grammar::xml_grammar,
    ///     parser::{language::EarleyParser, Parser},
    /// };
    ///
    /// let xml_tokens = HashSet::from(["id".to_string(), "text".to_string()]);
    /// let grammar = xml_grammar();
    /// let ep = EarleyParser::new(grammar, "start", xml_tokens, true);
    /// let forest = ep.parse_forest(b"<html>Text</html>").unwrap();
    /// println!("Len: {}", forest.len());
    /// for tree in forest {
    ///     // println!("Parsed tree:\n{}", tree);
    ///     assert_eq!(tree.to_bytes(), b"<html>Text</html>");
    /// }
    /// ```
    fn parse_forest(&self, text: &[u8]) -> Result<Vec<Arc<DerivationTree>>, String> {
        let (cursor, forest) = self.parse_prefix(text);

        if cursor < text.len() {
            return Err(format!(
                "Syntax error at position {}: unparsed text '{}'",
                cursor,
                String::from_utf8_lossy(&text[cursor..])
            ));
        }

        let pruned_trees = forest
            .into_iter()
            .map(|tree| self.prune_tree(tree))
            .collect();
        Ok(pruned_trees)
    }

    fn grammar(&self) -> &Grammar {
        &self.grammar
    }

    fn start_symbol(&self) -> &str {
        &self.start_symbol
    }

    fn tokens(&self) -> &HashSet<String> {
        &self.tokens
    }

    fn coalesce_tokens(&self) -> bool {
        self.coalesce_tokens
    }

    fn parse_regions(&self, text: &[u8]) -> Result<HashMap<String, HashSet<Region>>, String> {
        let chart = self.chart_parse(text);
        let mut regions: HashMap<String, HashSet<Region>> = HashMap::new();
        for column in chart {
            for state in column.states {
                // FIXME: use a new function to exclude
                // Do not create an unuseful temp variable
                if !self.is_excluded(&Symbol::NonTerminal {
                    label: state.name.clone(),
                }) && state.is_finished()
                    && state.e_col_idx - state.s_col_idx > 1
                {
                    let region = Region {
                        start: state.s_col_idx,
                        end: state.e_col_idx,
                    };

                    regions
                        .entry(state.name.clone())
                        .or_default()
                        .insert(region);
                }
            }
        }
        Ok(regions)
    }
}

impl LanguageParser for EarleyParser {
    fn parse_prefix(&self, text: &[u8]) -> (usize, Vec<Arc<DerivationTree>>) {
        let chart = self.chart_parse(text);

        if let Some(last_column) = chart.last() {
            for state in last_column.states.iter().rev() {
                if state.name == self.start_symbol && state.is_finished() && state.s_col_idx == 0 {
                    let mut memo = HashMap::new();
                    let trees = self.extract_trees(&chart, state, &mut memo);
                    return (text.len(), trees);
                }
            }
        }

        for i in (0..chart.len()).rev() {
            if !chart[i].states.is_empty() {
                return (i, vec![]);
            }
        }

        (0, vec![])
    }
}
