//! Simple Non-Terminal

use std::{any::Any, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

#[allow(deprecated)]
use crate::{NonTerminalKind, Symbol};

/// Bytes terminal.
/// TODO: support big/little-endian.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct NonTerminal {
    label: String,
}

impl NonTerminal {
    /// Create a new Non Terminal
    pub fn new(name: &str) -> Self {
        Self {
            label: name.to_string(),
        }
    }
}

#[typetag::serde]
impl NonTerminalKind for NonTerminal {
    fn display(&self) -> String {
        format!("<{}>", self.label)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_dyn(&self, other: &dyn NonTerminalKind) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<Self>() {
            self == other_val
        } else {
            false
        }
    }

    fn hash_dyn(&self, state: &mut dyn Hasher) {
        state.write(b"NonTerminal");
        state.write(&self.label.as_bytes());
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// Create a NonTerminal.
pub fn nt(label: &str) -> Symbol {
    Symbol::NonTerminal {
        kind: Arc::new(NonTerminal::new(label)),
    }
}
