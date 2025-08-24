use std::{any::Any, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::symbol::{Symbol, terminals::TerminalKind};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct BytesTerminal {
    value: Vec<u8>,
    length: usize,
}

impl BytesTerminal {
    /// Create a new BytesTerminal.
    pub fn new_from_len(length: usize) -> Self {
        Self {
            value: vec![],
            length,
        }
    }

    /// Create a new BytesTerminal.
    pub fn new_from_val(value: &[u8]) -> Self {
        Self {
            value: value.to_vec(),
            length: value.len(),
        }
    }
}

impl TerminalKind for BytesTerminal {
    fn display_terminal(&self) -> String {
        format!("Bytes[{}]", self.length)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.length {
            Err(format!("Not enough data for Bytes[{}]", self.length))
        } else {
            Ok(self.value[..self.length].to_vec())
        }
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
        state.write(b"BytesTerminal");
        state.write(&self.value);
    }
}

/// Create a Binary Bytes Terminal.
pub fn t_bytes(size: usize) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(BytesTerminal::new_from_len(size)),
    }
}

/// Create a Binary Bytes Terminal.
pub fn t_bytes_val(val: &[u8]) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(BytesTerminal::new_from_val(val)),
    }
}
