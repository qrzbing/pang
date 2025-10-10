//! Symbol contains Terminal or NonTerminal.

use std::{
    any::Any,
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

pub mod nonterminals;
pub use nonterminals::*;
pub mod terminals;
pub use terminals::*;
pub mod traits;
pub use traits::HasLength;

/// Decode error types.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// Incomplete data
    Incomplete(&'static str),
    /// Invalid data format
    Invalid(Cow<'static, str>),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Incomplete(msg) => {
                write!(f, "Incomplete data: {}", msg)
            }
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
        /// Name of the terminal.
        label: Option<String>,
        /// Terminal points to a TerminalKind.
        kind: Arc<dyn TerminalKind>,
    },
    /// NonTerminal can be expanded by other symbols.
    NonTerminal {
        /// NonTerminal Kind.
        kind: Arc<dyn NonTerminalKind>,
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
            Symbol::NonTerminal { kind, .. } => kind.label(),
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
            Symbol::NonTerminal { kind } => write!(f, "{}", kind.display()),
            Symbol::Terminal { label, kind } => {
                if let Some(label) = label {
                    write!(f, "{}", label)
                } else {
                    write!(f, "{}", kind.display_terminal())
                }
            }
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // Compare non-terminal
            (Symbol::NonTerminal { kind: self_kind }, Symbol::NonTerminal { kind: other_kind }) => {
                self_kind.eq_dyn(&**other_kind)
            }
            // Compare terminal
            (
                Symbol::Terminal {
                    label: self_label,
                    kind: self_kind,
                },
                Symbol::Terminal {
                    label: other_label,
                    kind: other_kind,
                },
            ) => self_label == other_label && self_kind.eq_dyn(other_kind.as_ref()),

            _ => false,
        }
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Symbol::Terminal { label, kind } => {
                0.hash(state);
                if let Some(label) = label {
                    label.hash(state);
                }
                kind.hash_dyn(state);
            }
            Symbol::NonTerminal { kind } => {
                1.hash(state);
                kind.hash_dyn(state);
            }
        }
    }
}

/// Display byte slice as a string.
#[inline]
pub fn display_u8(inp: &[u8]) -> String {
    inp.iter()
        .map(|&byte| match byte {
            b'\n' => "\\n".to_string(),
            b'\r' => "\\r".to_string(),
            b'\t' => "\\t".to_string(),

            b if (b as char).is_ascii_graphic() => (b as char).to_string(),

            _ => format!("\\x{:02x}", byte),
        })
        .collect()
}
