//! Symbol contains Terminal or NonTerminal.

use std::{
    any::Any,
    collections::HashMap,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub mod terminals;
pub use terminals::{
    TerminalKind,
    ber_length::{BerLengthTerminal, t_ber, t_ber_val},
    bits::{BitsTerminal, t_bis_val, t_bits},
    bytes::{BytesTerminal, t_bytes, t_bytes_val},
    dynamic::{DynamicTerminal, t_dyn, t_dyn_val},
    literal::{LiteralTerminal, t},
};
pub mod traits;
pub use traits::HasLength;

/// Decode error types.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// Incomplete data
    Incomplete(&'static str),
    /// Invalid data format
    Invalid(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Incomplete(msg) => write!(f, "Incomplete data: {}", msg),
            DecodeError::Invalid(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for DecodeError {}

/// A shared state for custom parsers.
#[derive(Debug, Clone, Default)]
pub struct SharedState {
    items: Arc<Mutex<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl SharedState {
    /// Create a new shared state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value to the shared state.
    pub fn set<T: 'static + Send + Sync>(&self, key: String, value: T) {
        self.items.lock().unwrap().insert(key, Box::new(value));
    }

    /// Get a value from the shared state.
    pub fn get<T: 'static + Send + Sync>(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        self.items
            .lock()
            .unwrap()
            .get(key)
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
    }
}

/// DecodeResult type alias
///
/// Returns a tuple of the decoded value and the remaining data.
pub type DecodeResult<'a, T> = Result<(&'a [u8], T), DecodeError>;

/// Symbol contains Terminal or NonTerminal.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl Symbol {
    /// Display a symbol in a human-readable format.
    pub fn display_symbol(&self) -> String {
        self.to_string()
    }

    /// Get the label of a NonTerminal symbol.
    pub fn label(&self) -> &str {
        match self {
            Symbol::NonTerminal { label } => label,
            _ => panic!("Cannot call .label() on a Terminal symbol"),
        }
    }

    /// Checks if the given symbol is a nonterminal.
    pub fn is_nonterminal(&self) -> bool {
        matches!(self, Symbol::NonTerminal { .. })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::NonTerminal { label } => write!(f, "<{}>", label),
            Symbol::Terminal { kind } => {
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
                1.hash(state);
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
