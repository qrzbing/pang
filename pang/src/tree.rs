//! Derivation tree is used in parser and generator.

use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};

use crate::grammar::{
    Symbol,
    TerminalKind::{Binary, Literal},
};

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
