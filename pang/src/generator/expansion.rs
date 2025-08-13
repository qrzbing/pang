use log::debug;

use super::*;

impl Generator {
    pub(super) fn expansion_to_children(&self, expansion: &Expansion) -> Vec<Arc<DerivationTree>> {
        self.expansion_invocations
            .set(self.expansion_invocations.get() + 1);

        // Only check the cache if the expansion is deterministic and cacheable.
        let cacheable = is_expansion_cacheable(expansion);
        if cacheable {
            if let Some(cached_children) = self.expansion_cache.borrow().get(expansion) {
                self.expansion_invocations_cached
                    .set(self.expansion_invocations_cached.get() + 1);
                return cached_children.clone();
            }
        }

        let result = expansion_to_children(expansion);
        if cacheable {
            self.expansion_cache
                .borrow_mut()
                .insert(expansion.clone(), result.clone());
        }
        result
    }

    #[allow(dead_code)]
    pub(super) fn choose_node_expansion(
        _node: DerivationTree,
        children_alternatives: Vec<Vec<Arc<DerivationTree>>>,
    ) -> u32 {
        return rand::random::<u32>() % children_alternatives.len() as u32;
    }

    /// Choose a random expansion for `node` and return it
    pub(super) fn expand_node_randomly(&self, node: &Arc<DerivationTree>) -> Arc<DerivationTree> {
        assert!(node.children.is_none(), "Node must be unexpanded");
        let label = match &node.symbol {
            Symbol::NonTerminal { label } => label,
            _ => panic!("Cannot expand a terminal symbol"),
        };
        let expansions = self
            .grammar
            .get(label)
            .expect("Non-terminal symbol not found in grammar");
        if expansions.is_empty() {
            panic!("No expansions found for symbol: {}", label);
        }
        let children_alternatives: Vec<_> = expansions
            .iter()
            .map(|exp| self.expansion_to_children(exp))
            .collect();
        let mut rng = rand::rng();
        let index = rng.random_range(0..children_alternatives.len());
        let chosen_children = children_alternatives[index].clone();
        new_node(node.symbol.clone(), Some(chosen_children), None)
    }

    /// Counts how many unexpanded symbols there are in a tree
    pub(super) fn possible_expansions(&self, node: &DerivationTree) -> u32 {
        match &node.children {
            None => 1,
            Some(children) => children
                .iter()
                .map(|child| self.possible_expansions(child))
                .sum(),
        }
    }

    /// If the tree has any non-expanded nodes, return true.
    pub(super) fn any_possible_expansions(&self, node: &DerivationTree) -> bool {
        if node.children.is_none() {
            return matches!(node.symbol, Symbol::NonTerminal { .. });
        }

        if let Some(children) = &node.children {
            children
                .iter()
                .any(|child| self.any_possible_expansions(child))
        } else {
            true
        }
    }

    /// Return index of subtree in `children` to be selected for expansion.
    /// Defaults to random.
    #[allow(dead_code)]
    pub(super) fn choose_tree_expansion(
        &self,
        _tree: &Arc<DerivationTree>,
        children: &mut Vec<Arc<DerivationTree>>,
    ) -> u32 {
        return rand::random::<u32>() % children.len() as u32;
    }

    /// Choose an unexpanded symbol in tree; expand it.
    /// Can be overloaded in subclasses.
    pub(super) fn expand_tree_once(
        &self,
        tree: &Arc<DerivationTree>,
        strategy: ExpansionStrategy,
    ) -> Arc<DerivationTree> {
        if tree.children.is_none() {
            return self.expand_node_with_strategy(tree, strategy);
        }

        // Find all expandable children
        let children = tree.children.as_ref().unwrap();
        let expandable_children_indices: Vec<usize> = children
            .iter()
            .enumerate()
            .filter(|(_i, child)| self.any_possible_expansions(child))
            .map(|(i, _child)| i)
            .collect();

        // No expandable children, return the original tree
        if expandable_children_indices.is_empty() {
            return tree.clone();
        }

        // Choose a random child to expand
        // TODO: In fuzzingbook, here use choose_tree_expansion,
        //       which will be overloaded by subclasses.
        let mut rng = rand::rng();
        let chosen_index_in_filtered_list = rng.random_range(0..expandable_children_indices.len());
        let original_index_to_expand = expandable_children_indices[chosen_index_in_filtered_list];

        // Expand the chosen child
        let mut new_children = children.clone();
        let child_to_expand = &new_children[original_index_to_expand];
        let expanded_child = self.expand_tree_once(child_to_expand, strategy);

        // Replace the original child with the expanded one
        new_children[original_index_to_expand] = expanded_child.into();
        new_node(tree.symbol.clone(), Some(new_children), None)
    }

    pub(super) fn expand_node_by_cost(
        &self,
        node: &Arc<DerivationTree>,
        strategy: CostStrategy,
    ) -> Arc<DerivationTree> {
        assert!(node.children.is_none(), "Node must be unexpanded");
        let label = match &node.symbol {
            Symbol::NonTerminal { label } => label,
            _ => panic!("Cannot expand a terminal symbol"),
        };

        // Find all expansions of the symbol
        let expansions = self.grammar.get(label).expect("Symbol not in grammar");
        debug!("expansions: {:?}", expansions);
        if expansions.is_empty() {
            panic!("No expansions found for symbol: {}", label);
        }

        // 2. Compute the cost of each expansion
        let alternatives: Vec<_> = expansions
            .iter()
            .map(|expansion| {
                let cost = self.expansion_cost(expansion);
                (expansion, cost)
            })
            .collect();
        debug!("Alternatives with costs: {:?}", alternatives);

        // 3. Choose the best expansion based on the chosen strategy
        let chosen_cost = match strategy {
            CostStrategy::Min => alternatives
                .iter()
                .map(|(_, cost)| *cost)
                .reduce(f64::min)
                .unwrap_or(f64::INFINITY),
            CostStrategy::Max => {
                let finite_max = alternatives.iter().map(|(_, cost)| *cost).reduce(f64::max);

                finite_max.unwrap_or(f64::INFINITY)
            }
        };
        debug!("Chosen cost: {}", chosen_cost);

        let best_expansions: Vec<_> = alternatives
            .into_iter()
            .filter(|(_, cost)| *cost == chosen_cost)
            .map(|(expansion, _)| expansion)
            .collect();
        debug!("Best expansions: {:?}", best_expansions);

        let mut rng = rand::rng();
        let chosen_expansion = best_expansions[rng.random_range(0..best_expansions.len())];

        let chosen_children = self.expansion_to_children(chosen_expansion);
        let final_children = self.process_chosen_children(chosen_children, chosen_expansion);

        new_node(node.symbol.clone(), Some(final_children), None)
    }

    pub(super) fn expand_node_min_cost(&self, node: &Arc<DerivationTree>) -> Arc<DerivationTree> {
        self.expand_node_by_cost(node, CostStrategy::Min)
    }

    pub(super) fn expand_node_max_cost(&self, node: &Arc<DerivationTree>) -> Arc<DerivationTree> {
        self.expand_node_by_cost(node, CostStrategy::Max)
    }

    pub(super) fn expand_node_with_strategy(
        &self,
        node: &Arc<DerivationTree>,
        strategy: ExpansionStrategy,
    ) -> Arc<DerivationTree> {
        match strategy {
            ExpansionStrategy::Random => self.expand_node_randomly(node),
            ExpansionStrategy::MinCost => self.expand_node_min_cost(node),
            ExpansionStrategy::MaxCost => self.expand_node_max_cost(node),
        }
    }

    pub(super) fn expand_tree_with_strategy(
        &self,
        mut tree: Arc<DerivationTree>,
        strategy: ExpansionStrategy,
        limit: Option<u32>,
    ) -> Arc<DerivationTree> {
        while self.any_possible_expansions(&tree) {
            if let Some(lim) = limit {
                if self.possible_expansions(&tree) >= lim {
                    break;
                }
            }
            tree = self.expand_tree_once(&tree, strategy);
        }
        tree
    }

    pub(super) fn expand_tree(&self, tree: Arc<DerivationTree>) -> Arc<DerivationTree> {
        let tree = self.expand_tree_with_strategy(
            tree,
            ExpansionStrategy::MaxCost,
            Some(self.min_nonterminals),
        );

        let tree = self.expand_tree_with_strategy(
            tree,
            ExpansionStrategy::Random,
            Some(self.max_nonterminals),
        );

        let tree = self.expand_tree_with_strategy(tree, ExpansionStrategy::MinCost, None);
        assert_eq!(
            self.possible_expansions(&tree),
            0,
            "Tree must be fully expanded"
        );

        tree
    }
}
