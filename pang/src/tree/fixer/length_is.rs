use std::sync::Arc;

use crate::{
    grammar::{BinaryKind, Grammar, Symbol, TerminalKind},
    parser::format::usize_to_ber_bytes,
    tree::{DerivationTree, TreeFixer, new_node},
};

/// LengthIsFixer is a [`TreeFixer`] that fixes the length of a node
/// based on the length of a value.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
///
/// use pang::{
///     generator::Generator,
///     grammar::asn1_tlv_grammar,
///     tree::{decoder::ber_to_usize, fixer::LengthIsFixer},
/// };
///
/// let grammar = asn1_tlv_grammar();
/// let generator = Generator::new(
///     grammar,
///     "asn1-tlv",
///     4,
///     6,
///     vec![Arc::new(LengthIsFixer::new())],
/// );
///
/// let tree = generator.generate_tree();
/// assert_eq!(tree.symbol.label(), "asn1-tlv");
///
/// let children = tree.children.as_ref().unwrap();
/// assert_eq!(children.len(), 3);
/// assert_eq!(tree.at(&[0]).unwrap().symbol.label(), "asn1-tlv-type");
/// assert_eq!(tree.at(&[1]).unwrap().symbol.label(), "asn1-tlv-len");
/// assert_eq!(tree.at(&[2]).unwrap().symbol.label(), "asn1-tlv-value");
///
/// let asn1_tlv_len = tree
///     .at(&[1])
///     .unwrap()
///     .decode(ber_to_usize)
///     .unwrap();
///
/// assert_eq!(asn1_tlv_len, children[2].to_bytes().len());
/// ```
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
    /// Fix TLV length based on [`Grammar`].
    ///
    /// # Example
    ///
    /// ```text
    /// Before:
    /// <asn1-tlv>
    /// ├── <asn1-tlv-type>
    /// │   └── [05]
    /// ├── <asn1-tlv-len>
    /// │   └── Dynamic: [0x00]
    /// └── <asn1-tlv-value>
    ///     └── Dynamic: [0x01, 0x02, 0x03, 0x04]
    ///
    /// After:
    /// <asn1-tlv>
    /// ├── <asn1-tlv-type>
    /// │   └── [05]
    /// ├── <asn1-tlv-len>
    /// │   └── Dynamic: [0x04]
    /// └── <asn1-tlv-value>
    ///     └── Dynamic: [0x01, 0x02, 0x03, 0x04]
    /// ```
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// use pang::{
    ///     grammar::{asn1_tlv_grammar, nt, t, t_dyn},
    ///     tree::{fixer::LengthIsFixer, new_node}
    /// };
    ///
    /// let grammar = asn1_tlv_grammar();
    /// let tree = new_node(
    ///     nt("asn1-tlv"),
    ///     Some(vec![
    ///         new_node(
    ///             nt("asn1-tlv-type"),
    ///             Some(vec![new_node(t(&[0x05]), Some(vec![]), None)]),
    ///             None,
    ///         ),
    ///         new_node(
    ///             nt("asn1-tlv-len"),
    ///             Some(vec![new_node(t_dyn(), Some(vec![]), Some(vec![0x00]))]),
    ///             None,
    ///         ),
    ///         new_node(
    ///             nt("asn1-tlv-value"),
    ///             Some(vec![new_node(t_dyn(), Some(vec![]), Some(vec![0x01, 0x02, 0x03, 0x04]))]),
    ///             None,
    ///         ),
    ///     ]),
    ///     None,
    /// );
    /// let tlv_len_orig = tree.at(&[1]).unwrap().all_terminals();
    /// assert_eq!(tlv_len_orig, vec![0x00]);
    ///
    /// let tree = tree.fix_tree(&grammar, &[Arc::new(LengthIsFixer::new())]);
    /// let tlv_len_fixed = tree.at(&[1]).unwrap().all_terminals();
    /// assert_eq!(tlv_len_fixed, vec![0x04]);
    /// ```
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
