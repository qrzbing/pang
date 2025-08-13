use std::sync::Arc;

use crate::{
    grammar::{BinaryKind, Grammar, Symbol, TerminalKind},
    parser::format::usize_to_ber_bytes,
    tree::{DerivationTree, TreeFixer, new_node},
};

/// LengthIsFixer is a [`TreeFixer`] that fixes the length of a node
/// based on the length of a value.
#[derive(Debug, Default)]
pub struct LengthIsFixer;

impl LengthIsFixer {
    /// Create a new [`LengthIsFixer`].
    pub fn new() -> Self {
        Self::default()
    }

    fn get_len_node_terminal_symbol<'a>(
        &self,
        grammar: &'a Grammar,
        len_label: &str,
    ) -> Option<&'a Symbol> {
        grammar
            .get(len_label)?
            .iter()
            .find_map(|exp| exp.symbols.first())
            .filter(|sym| matches!(sym, Symbol::Terminal { .. }))
    }

    fn update_terminal_value(
        &self,
        node: Arc<DerivationTree>,
        new_value: Vec<u8>,
    ) -> Arc<DerivationTree> {
        if let Symbol::Terminal { .. } = &node.symbol {
            return new_node(node.symbol.clone(), node.children.clone(), Some(new_value));
        }

        if let Some(children) = &node.children {
            let mut new_children = Vec::with_capacity(children.len());
            let mut updated = false;
            for child in children {
                if !updated {
                    let new_child = self.update_terminal_value(child.clone(), new_value.clone());
                    if !Arc::ptr_eq(child, &new_child) {
                        updated = true;
                    }
                    new_children.push(new_child);
                } else {
                    new_children.push(child.clone());
                }
            }
            if updated {
                return new_node(node.symbol.clone(), Some(new_children), node.value.clone());
            }
        }
        node
    }
}

impl TreeFixer for LengthIsFixer {
    fn fix(&self, grammar: &Grammar, node: Arc<DerivationTree>) -> Arc<DerivationTree> {
        // Skip nodes without children
        let children = match &node.children {
            Some(c) if !c.is_empty() => c,
            _ => return node,
        };

        let mut new_children = children.clone();
        let mut children_changed = false;

        for value_candidate_node in children.iter() {
            if let Symbol::NonTerminal {
                label: value_candidate_label,
            } = &value_candidate_node.symbol
            {
                if let Some(value_expansions) = grammar.get(value_candidate_label) {
                    for value_exp in value_expansions {
                        if let Some(len_label) =
                            value_exp.options.get("length_is").and_then(|v| v.as_str())
                        {
                            let value_bytes = value_candidate_node.to_bytes();
                            if let Some(len_node_idx) = children.iter().position(|c| {
                                if let Symbol::NonTerminal { label } = &c.symbol {
                                    label == len_label
                                } else {
                                    false
                                }
                            }) {
                                let len_node_to_update = &children[len_node_idx];
                                let len_bytes_encoded = match value_exp
                                    .options
                                    .get("length_type")
                                    .and_then(|v| v.as_str())
                                {
                                    Some("ber") => usize_to_ber_bytes(value_bytes.len()),
                                    _ => {
                                        let size_in_bytes = if let Some(Symbol::Terminal {
                                            kind: TerminalKind::Binary(BinaryKind::Bytes { size }),
                                        }) =
                                            self.get_len_node_terminal_symbol(grammar, len_label)
                                        {
                                            size
                                        } else {
                                            &4
                                        };
                                        let mut bytes_val =
                                            (value_bytes.len() as u64).to_le_bytes().to_vec();
                                        if let Some("big") =
                                            value_exp.options.get("endian").and_then(|v| v.as_str())
                                        {
                                            bytes_val =
                                                (value_bytes.len() as u64).to_be_bytes().to_vec();
                                        }
                                        bytes_val.truncate(*size_in_bytes);
                                        bytes_val
                                    }
                                };
                                let new_len_node = self.update_terminal_value(
                                    len_node_to_update.clone(),
                                    len_bytes_encoded,
                                );
                                new_children[len_node_idx] = new_len_node;
                                children_changed = true;

                                break;
                            }
                        }
                    }
                }
            }
        }

        if children_changed {
            Arc::new(DerivationTree {
                symbol: node.symbol.clone(),
                children: Some(new_children),
                value: node.value.clone(),
            })
        } else {
            node
        }
    }
}
