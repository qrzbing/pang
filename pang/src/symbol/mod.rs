//! TODO

use std::{
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    sync::Arc,
};

///
pub mod terminals;
use terminals::TerminalKind;

/// Decode error types.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// Incomplete data
    Incomplete,
    /// Invalid data format
    InvalidData(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Incomplete => write!(f, "Incomplete data"),
            DecodeError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for DecodeError {}

/// DecodeResult type alias
///
/// Returns a tuple of the decoded value and the remaining data.
pub type DecodeResult<'a, T> = Result<(&'a [u8], T), DecodeError>;

/// Symbol contains Terminal or NonTerminal.
#[derive(Clone, Debug)]
pub enum Symbol {
    /// Terminal can not be expanded.
    Terminal {
        /// Terminal points to a TerminalKind.
        kind: Arc<dyn TerminalKind>,
    },
    /// NonTerminal can be expanded by other symbols.
    NonTerminal {
        /// NonTerminal has a label as its name.
        label: String,
    },
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::NonTerminal { label } => write!(f, "<{}>", label),
            Self::Terminal { kind } => {
                write!(f, "{}", kind.display_terminal())
            }
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare non-terminal
            (
                Symbol::NonTerminal { label: self_label },
                Symbol::NonTerminal { label: other_label },
            ) => self_label == other_label,

            // Compare terminal
            (Symbol::Terminal { kind: self_kind }, Symbol::Terminal { kind: other_kind }) => {
                self_kind.eq_dyn(other_kind.as_ref())
            }

            _ => false,
        }
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Symbol::Terminal { kind } => {
                0.hash(state);
                kind.hash_dyn(state);
            }
            Symbol::NonTerminal { label } => {
                1.hash(state); // NonTerminal 的标识符
                label.hash(state);
            }
        }
    }
}

/// Create a NonTerminal.
pub fn nt(label: &str) -> Symbol {
    Symbol::NonTerminal {
        label: label.to_string(),
    }
}
