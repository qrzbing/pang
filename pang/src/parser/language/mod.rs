//! Language Parser is for context-free grammar like JSON, XML, etc.

use std::sync::Arc;

use crate::{
    grammar::{Symbol, TerminalKind, t},
    parser::Parser,
    tree::{DerivationTree, new_node},
};

mod earley;
pub use earley::EarleyParser;

const PHONY_START_SYMBOL: &str = "<>";

/// Language Parser contains some methods for context-free grammar parsing.
pub trait LanguageParser: Parser {
    /// The core parsing method to be implemented by a concrete parser (e.g., Earley).
    ///
    /// It should parse the longest possible prefix of `text` that conforms to the grammar.
    ///
    /// # Returns
    /// A tuple `(cursor, forest)` where:
    /// * `cursor` is the index in `text` up to which parsing was successful.
    /// * `forest` is a vector of possible derivation trees for the parsed prefix.
    fn parse_prefix(&self, text: &[u8]) -> (usize, Vec<Arc<DerivationTree>>);

    /// Simplifies a derivation tree into a more abstract syntax tree (AST).
    ///
    /// This method performs two main actions:
    /// 1. **Coalescing**: Merges adjacent terminal symbols if `coalesce_tokens` is true.
    /// 2. **Token Pruning**: If a node's symbol is in `self.tokens`, its children
    ///    are replaced by a single terminal node containing the derived string.
    fn prune_tree(&self, tree: Arc<DerivationTree>) -> Arc<DerivationTree> {
        // Handle and remove the phony start symbol, if it was used.
        if let Symbol::NonTerminal { label } = &tree.symbol {
            if label == PHONY_START_SYMBOL {
                assert_eq!(
                    tree.children.as_ref().map_or(0, |c| c.len()),
                    1,
                    "Phony start symbol must have exactly one child."
                );
                let single_child = Arc::clone(&tree.children.as_ref().unwrap()[0]);
                return self.prune_tree(single_child);
            }
        }

        let mut new_children = tree.children.as_ref().cloned().unwrap_or_default();

        if self.coalesce_tokens() {
            new_children = self.coalesce(new_children);
        }

        // If the symbol is a designated token, replace its subtree with its string value.
        if let Symbol::NonTerminal { label } = &tree.symbol {
            if self.tokens().contains(label) {
                let derived_string =
                    &new_node(tree.symbol.clone(), Some(new_children), None).all_terminals();
                let terminal_child = new_node(t(&derived_string), Some(vec![]), None);
                return new_node(
                    Symbol::NonTerminal {
                        label: label.clone(),
                    },
                    Some(vec![terminal_child]),
                    None,
                );
            }
        }

        // Otherwise, recurse on children.
        let pruned_children = new_children
            .into_iter()
            .map(|c| self.prune_tree(c))
            .collect();
        new_node(tree.symbol.clone(), Some(pruned_children), None)
    }

    /// Merges consecutive terminal nodes in a list of children into single nodes.
    fn coalesce(&self, children: Vec<Arc<DerivationTree>>) -> Vec<Arc<DerivationTree>> {
        let mut new_children = Vec::new();
        let mut current_terminal = Vec::new();

        for child in children {
            match &child.symbol {
                Symbol::Terminal { kind } => match kind {
                    TerminalKind::Literal(value) => {
                        if value.is_empty() {
                            // Skip empty terminals often used to represent epsilon-productions
                        } else {
                            current_terminal.extend_from_slice(value)
                        }
                    }
                    _ => panic!("Do not use parser on Bytes / Bits"),
                },
                Symbol::NonTerminal { .. } => {
                    // When a non-terminal is found, push any accumulated terminal first.
                    if !current_terminal.is_empty() {
                        new_children.push(new_node(t(&current_terminal), Some(vec![]), None));
                        current_terminal.clear();
                    }
                    new_children.push(child);
                }
            }
        }

        // Push any remaining accumulated terminal.
        if !current_terminal.is_empty() {
            new_children.push(new_node(t(&current_terminal), Some(vec![]), None));
        }

        new_children
    }
}
