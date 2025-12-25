//! LiteralTerminal

use std::{
    any::Any,
    hash::{Hash, Hasher},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::symbol::{Symbol, terminals::TerminalKind};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct LiteralTerminal {
    ///
    pub value: String,
}

impl LiteralTerminal {
    /// Create a new LiteralTerminal.
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[typetag::serde]
impl TerminalKind for LiteralTerminal {
    fn display_terminal(&self) -> String {
        format!("{}", self.value)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        Ok(self.value.as_bytes().to_vec())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_dyn(&self, other: &dyn TerminalKind) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<Self>() {
            self == other_val
        } else {
            false
        }
    }

    fn hash_dyn(&self, state: &mut dyn Hasher) {
        state.write(b"LiteralTerminal");
        state.write(self.value.as_bytes());
    }
}

/// Create a Literal Terminal.
pub fn t(value: &str) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(LiteralTerminal::new(value.to_string())),
    }
}

/// Create a Literal Terminal with label.
pub fn tl(label: &str, value: &str) -> Symbol {
    Symbol::Terminal {
        label: Some(label.to_string()),
        kind: Arc::new(LiteralTerminal::new(value.to_string())),
    }
}
