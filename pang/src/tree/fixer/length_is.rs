use std::sync::Arc;

use log::debug;

use crate::{
    grammar::Grammar,
    symbol::Symbol,
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
/// use libafl_bolts::rands::StdRand;
///
/// use pang::{language::asn1_tlv_lang, tree::fixer::LengthIsFixer};
///
/// let mut rng = StdRand::with_seed(0);
///
/// let tree = asn1_tlv_lang().grammar.generate_combinator(
///     "asn1-tlv",
///     &mut rng,
///     &[Arc::new(LengthIsFixer::new())],
/// );
///
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
///     .first_terminal_kind()
///     .unwrap()
///     .as_has_length()
///     .unwrap()
///     .as_length()
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
    ///     language::asn1_tlv_lang,
    ///     symbol::{
    ///         nt,
    ///         terminals::{ber_length::t_ber, bytes::t_bytes_val, dynamic::t_dyn_value},
    ///     },
    ///     tree::{fixer::LengthIsFixer, new_node},
    /// };
    ///
    /// let grammar = asn1_tlv_lang().grammar;
    /// let tree = new_node(
    ///     nt("asn1-tlv"),
    ///     Some(vec![
    ///         new_node(
    ///             nt("asn1-tlv-type"),
    ///             Some(vec![new_node(t_bytes_val(&[0x05]), Some(vec![]))]),
    ///         ),
    ///         new_node(
    ///             nt("asn1-tlv-len"),
    ///             Some(vec![new_node(t_ber(), Some(vec![]))]),
    ///         ),
    ///         new_node(
    ///             nt("asn1-tlv-value"),
    ///             Some(vec![new_node(
    ///                 t_dyn_value(&[0x01, 0x02, 0x03, 0x04]),
    ///                 Some(vec![]),
    ///             )]),
    ///         ),
    ///     ]),
    /// );
    /// let tlv_len_orig = tree.at(&[1]).unwrap().all_terminals();
    /// assert_eq!(tlv_len_orig, "0");
    /// println!("Original tree: {}", tree);
    ///
    /// let tree = tree.fix_tree(&grammar, &[Arc::new(LengthIsFixer::new())]);
    /// let tlv_len_fixed = tree.at(&[1]).unwrap().all_terminals();
    /// assert_eq!(tlv_len_fixed, "4");
    /// ```
    fn fix(&self, grammar: &Grammar, node: Arc<DerivationTree>) -> Arc<DerivationTree> {
        // Skip Terminal and NonTerminal without children)
        let (Some(children), Symbol::NonTerminal { label }) = (&node.children, &node.symbol) else {
            return node;
        };

        let Some(..) = grammar.get(label) else {
            return node;
        };

        let mut new_children = children.clone();
        let mut has_changed = false;

        for value_candidate_node in children.iter() {
            let Symbol::NonTerminal { label: value_label } = &value_candidate_node.symbol else {
                continue;
            };

            if let Some(value_expansions) = grammar.get(value_label) {
                for value_exp in value_expansions {
                    if let Some(len_label_arc) = value_exp.options.get("length_provider") {
                        if let Some(len_label) = len_label_arc.downcast_ref::<String>() {
                            debug!("Fix length of {} to {}", value_label, len_label);
                            let actual_length = value_candidate_node.to_bytes().len();

                            if let Some(len_node_idx) =
                                children.iter().position(|c| c.symbol.label() == len_label)
                            {
                                let original_len_terminal = children[len_node_idx]
                                    .first_terminal_kind()
                                    .expect("Length node must contain a terminal");

                                if let Some(has_length_trait_obj) =
                                    original_len_terminal.as_has_length()
                                {
                                    // Use the `HasLength` trait to convert the length to the actual value.
                                    let new_terminal_kind =
                                        has_length_trait_obj.from_length(actual_length);

                                    // Generate a new terminal node with the new length.
                                    let new_len_leaf = new_node(
                                        Symbol::Terminal {
                                            kind: new_terminal_kind,
                                        },
                                        Some(vec![]),
                                    );
                                    let new_len_node = new_node(
                                        children[len_node_idx].symbol.clone(),
                                        Some(vec![new_len_leaf]),
                                    );

                                    new_children[len_node_idx] = new_len_node;
                                    has_changed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if has_changed {
                break;
            }
        }

        if has_changed {
            new_node(node.symbol.clone(), Some(new_children))
        } else {
            node
        }
    }
}
