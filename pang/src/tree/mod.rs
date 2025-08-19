//! Derivation tree is used in parser and generator.

use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::grammar::{
    Grammar, Symbol,
    TerminalKind::{Binary, Literal},
};

pub mod decoder;
use decoder::{CustomDecoderFn, DecodeError, NodeValue};
pub mod fixer;
pub use fixer::TreeFixer;

/// DerivationTree is designed to represent for grammar,
///
/// DerivationTree has a Symbol and its children.
///
/// - For literal grammar, value will be None.
/// - For binary format grammar, value will be a vec of bytes.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DerivationTree {
    /// Symbol of the node.
    pub symbol: Symbol,

    /// Children of this node in the derivation tree.
    /// - For `Symbol::NonTerminal`:
    ///   - `None`: Need to expand.
    ///   - `Some(vec![...])`: Expanded and has children.
    /// - For `Symbol::Terminal`:
    ///   - `Some(vec![...])`: Terminal symbol, no children
    pub children: Option<Vec<Arc<DerivationTree>>>,

    /// Value of the terminal symbol.
    pub value: Option<Vec<u8>>,
}

impl DerivationTree {
    fn format_value(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, ": [")?;
            for (i, byte) in value.iter().take(16).enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "0x{:02X}", byte)?;
            }
            if value.len() > 16 {
                write!(f, ", ...")?;
            }
            write!(f, "]")?;

            let lossy_string = String::from_utf8_lossy(value);
            if !lossy_string.chars().all(char::is_control) {
                write!(f, " // \"{}\"", lossy_string.escape_default())?;
            }
        }
        Ok(())
    }

    fn display_recursive(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefix: &str,
        is_last: bool,
    ) -> fmt::Result {
        write!(f, "{}", prefix)?;
        write!(f, "{}", if is_last { "└── " } else { "├── " })?;
        write!(f, "{}", self.symbol.display_symbol())?;
        self.format_value(f)?;
        writeln!(f)?;
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        if let Some(children) = &self.children {
            let num_children = children.len();
            for (i, child) in children.iter().enumerate() {
                child.display_recursive(f, &new_prefix, i == num_children - 1)?;
            }
        }
        Ok(())
    }

    /// Converts the tree to a `Vec<u8>`, replace nonterminal symbols with their label.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::{
    ///     grammar::{nt, t},
    ///     tree::new_node,
    /// };
    ///
    /// let tree = new_node(
    ///     nt("start"),
    ///     Some(vec![new_node(
    ///         nt("expr"),
    ///         Some(vec![
    ///             new_node(nt("expr"), None, None),
    ///             new_node(t(b"+"), Some(vec![]), None),
    ///             new_node(nt("expr"), None, None),
    ///         ]),
    ///         None,
    ///     )]),
    ///     None,
    /// );
    ///
    /// let result = tree.all_terminals();
    /// assert_eq!(result, b"<expr>+<expr>");
    /// ```
    pub fn all_terminals(&self) -> Vec<u8> {
        match &self.symbol {
            Symbol::Terminal { kind } => match kind {
                Literal(value) => value.clone(),
                Binary(..) => self.value.clone().unwrap_or_default(),
            },
            Symbol::NonTerminal { label } => match &self.children {
                Some(children) => children
                    .iter()
                    .flat_map(|child_node| child_node.all_terminals())
                    .collect(),
                None => format!("<{}>", label.clone()).into(),
            },
        }
    }

    /// Converts the tree to a `Vec<u8>`, replace nonterminal symbols by empty Vec.
    pub fn to_bytes(&self) -> Vec<u8> {
        if let Some(children) = &self.children {
            if !children.is_empty() {
                return children.iter().flat_map(|child| child.to_bytes()).collect();
            }
        }
        match &self.symbol {
            Symbol::Terminal { kind } => match kind {
                Literal(value) => value.clone(),
                Binary(..) => self.value.clone().unwrap_or_default(),
            },
            Symbol::NonTerminal { .. } => Vec::new(),
        }
    }

    /// Converts the tree to a String.
    pub fn to_string(&self) -> String {
        let bytes = self.to_bytes();
        String::from_utf8_lossy(&bytes).to_string()
    }

    /// Get the node at a given path.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::{
    ///     grammar::{nt, t},
    ///     tree::new_node,
    /// };
    /// let tree = new_node(
    ///     nt("start"),
    ///     Some(vec![new_node(
    ///         nt("expr"),
    ///         Some(vec![
    ///             new_node(nt("expr"), None, None),
    ///             new_node(t(b"+"), Some(vec![]), None),
    ///             new_node(nt("expr"), None, None),
    ///         ]),
    ///         None,
    ///     )]),
    ///     None,
    /// );
    ///
    /// assert_eq!(tree.at(&[]), Ok(tree.clone()));
    ///
    /// assert_eq!(tree.at(&[0]), Ok(new_node(
    ///     nt("expr"),
    ///     Some(vec![
    ///         new_node(nt("expr"), None, None),
    ///         new_node(t(b"+"), Some(vec![]), None),
    ///         new_node(nt("expr"), None, None),
    ///     ]),
    ///     None,
    /// )));
    ///
    /// assert_eq!(tree.at(&[0, 2]), Ok(new_node(nt("expr"), None, None)));
    ///
    /// assert_eq!(tree.at(&[1]), Err("Invalid path: child index out of bounds"));
    ///
    /// assert_eq!(tree.at(&[0, 2, 3]), Err("Invalid path: node has no children"));
    /// ```
    pub fn at(self: &Arc<Self>, path: &[usize]) -> Result<Arc<DerivationTree>, &'static str> {
        let mut current_node = self;
        for &index in path {
            let children = current_node
                .children
                .as_ref()
                .ok_or("Invalid path: node has no children")?;
            current_node = children
                .get(index)
                .ok_or("Invalid path: child index out of bounds")?;
        }
        Ok(current_node.clone())
    }

    /// Modify a node by path with a function.
    ///
    /// The modification is specified by a closure `f` which takes the target node
    /// and returns the node that should replace it.
    pub fn modify_by_path<F>(
        self: &Arc<Self>,
        path: &[usize],
        f: F,
    ) -> Result<Arc<DerivationTree>, &'static str>
    where
        F: FnOnce(&Arc<DerivationTree>) -> Arc<DerivationTree>,
    {
        if path.is_empty() {
            return Ok(f(self));
        }
        let child_index = path[0];
        let remaining_path = &path[1..];
        if let Some(children) = &self.children {
            if child_index >= children.len() {
                return Err("Invalid path: child index out of bounds");
            }

            // Recursively replace child node.
            let modified_child = children[child_index].modify_by_path(remaining_path, f)?;

            // No modification
            if Arc::ptr_eq(&children[child_index], &modified_child) {
                return Ok(self.clone());
            }

            // Create a new children vector with modified child
            let mut new_children = children.clone();
            new_children[child_index] = modified_child;

            Ok(Arc::new(DerivationTree {
                symbol: self.symbol.clone(),
                children: Some(new_children),
                value: self.value.clone(),
            }))
        } else {
            Err("Invalid path: node has no children to traverse")
        }
    }

    /// Replace a node by path
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::{
    ///     grammar::{nt, t},
    ///     tree::new_node,
    /// };
    /// let tree = new_node(
    ///     nt("start"),
    ///     Some(vec![new_node(
    ///         nt("expr"),
    ///         Some(vec![
    ///             new_node(nt("expr"), None, None),
    ///             new_node(t(b"+"), Some(vec![]), None),
    ///             new_node(nt("expr"), None, None),
    ///         ]),
    ///         None,
    ///     )]),
    ///     None,
    /// );
    /// let replace_node = new_node(t(b"number"), None, Some(b"123".to_vec()));
    /// let tree = tree.replace_by_path(&[0, 2], replace_node).unwrap();
    /// assert_eq!(tree, new_node(
    ///     nt("start"),
    ///     Some(vec![new_node(
    ///         nt("expr"),
    ///         Some(vec![
    ///             new_node(nt("expr"), None, None),
    ///             new_node(t(b"+"), Some(vec![]), None),
    ///             new_node(t(b"number"), None, Some(b"123".to_vec())),
    ///         ]),
    ///         None,
    ///     )]),
    ///     None,
    /// ));
    /// ```
    pub fn replace_by_path(
        self: &Arc<Self>,
        path: &[usize],
        new_node: Arc<DerivationTree>,
    ) -> Result<Arc<DerivationTree>, &'static str> {
        self.modify_by_path(path, |_| new_node)
    }

    /// Find the first path of a symbol in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::{
    ///     grammar::{nt, t},
    ///     tree::new_node,
    /// };
    /// let tree = new_node(
    ///     nt("start"),
    ///     Some(vec![new_node(
    ///         nt("expr"),
    ///         Some(vec![
    ///             new_node(nt("expr"), None, None),
    ///             new_node(t(b"+"), Some(vec![]), None),
    ///             new_node(nt("expr"), None, None),
    ///         ]),
    ///         None,
    ///     )]),
    ///     None,
    /// );
    /// assert_eq!(tree.find_first_path(&nt("expr")), Some(vec![0]));
    /// ```
    pub fn find_first_path(self: &Arc<Self>, symbol: &Symbol) -> Option<Vec<usize>> {
        let mut path = Vec::new();

        if self.find_first_recursive(symbol, &mut path) {
            Some(path)
        } else {
            None
        }
    }

    fn find_first_recursive(&self, symbol: &Symbol, current_path: &mut Vec<usize>) -> bool {
        if self.symbol == *symbol {
            return true;
        }

        if let Some(children) = &self.children {
            for (i, child) in children.iter().enumerate() {
                current_path.push(i);

                if child.find_first_recursive(symbol, current_path) {
                    return true;
                }

                current_path.pop();
            }
        }

        false
    }

    /// Find all paths of a symbol in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use pang::{
    ///     grammar::{nt, t},
    ///     tree::new_node,
    /// };
    /// let tree = new_node(
    ///     nt("start"),
    ///     Some(vec![new_node(
    ///         nt("expr"),
    ///         Some(vec![
    ///             new_node(nt("expr"), None, None),
    ///             new_node(t(b"+"), Some(vec![]), None),
    ///             new_node(nt("expr"), None, None),
    ///         ]),
    ///         None,
    ///     )]),
    ///     None,
    /// );
    /// assert_eq!(tree.find_all_paths(&nt("expr")), vec![vec![0], vec![0, 0], vec![0, 2]]);
    /// ```
    pub fn find_all_paths(self: &Arc<Self>, symbol: &Symbol) -> Vec<Vec<usize>> {
        let mut results = Vec::new();
        let mut current_path = Vec::new();
        self.find_all_recursive(symbol, &mut current_path, &mut results);
        results
    }

    fn find_all_recursive(
        &self,
        symbol: &Symbol,
        current_path: &mut Vec<usize>,
        results: &mut Vec<Vec<usize>>,
    ) {
        if self.symbol == *symbol {
            results.push(current_path.clone());
        }

        if let Some(children) = &self.children {
            for (i, child) in children.iter().enumerate() {
                current_path.push(i);
                child.find_all_recursive(symbol, current_path, results);
                current_path.pop();
            }
        }
    }

    /// Fix the derivation tree using a list of [`TreeFixer`].
    pub fn fix_tree(
        self: &Arc<Self>,
        grammar: &Grammar,
        fixers: &[Arc<dyn TreeFixer>],
    ) -> Arc<DerivationTree> {
        let mut fixed_node = if let Some(children) = &self.children {
            let mut changed = false;
            // Fix children first
            let fixed_children = children
                .iter()
                .map(|c| {
                    let fixed_child = c.fix_tree(grammar, fixers);
                    // Check if the child was changed
                    if !Arc::ptr_eq(c, &fixed_child) {
                        changed = true;
                    }

                    fixed_child
                })
                .collect();

            if changed {
                // Create a new node if children were potentially changed
                new_node(
                    self.symbol.clone(),
                    Some(fixed_children),
                    self.value.clone(),
                )
            } else {
                self.clone()
            }
        } else {
            // No children, no recursive call needed
            self.clone()
        };

        for fixer in fixers {
            fixed_node = fixer.fix(grammar, fixed_node);
        }

        fixed_node
    }

    /// Get the value of the node as [`NodeValue`].
    pub fn value(&self) -> Option<NodeValue> {
        let current_node_value = self.value.as_ref().map(|v| NodeValue::new(v.as_slice()));

        // If current node has value, return it.
        if let Some(value) = current_node_value {
            return Some(value);
        }

        // If current node has no value and has one child, return the value of the child.
        if let Some(children) = &self.children {
            if children.len() == 1 {
                return children[0].value();
            }
        }

        // TODO: handle other cases.
        None
    }

    /// Decode a tree to user-defined type.
    pub fn decode<'a, T>(&'a self, decoder: CustomDecoderFn<'a, T>) -> Result<T, DecodeError> {
        if let Some(value) = &self.value() {
            value.decode(decoder)
        } else {
            Err(DecodeError::InvalidData(
                "Cannot decode a non-terminal node without value",
            ))
        }
    }
}

impl fmt::Display for DerivationTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol.display_symbol())?;
        self.format_value(f)?;
        writeln!(f)?;
        if let Some(children) = &self.children {
            let num_children = children.len();
            for (i, child) in children.iter().enumerate() {
                child.display_recursive(f, "", i == num_children - 1)?;
            }
        }

        Ok(())
    }
}

/// Create a new node of derivation tree.
pub fn new_node(
    symbol: Symbol,
    children: Option<Vec<Arc<DerivationTree>>>,
    value: Option<Vec<u8>>,
) -> Arc<DerivationTree> {
    Arc::new(DerivationTree {
        symbol,
        children,
        value,
    })
}
