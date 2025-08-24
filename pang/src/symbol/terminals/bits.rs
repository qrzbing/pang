use std::{any::Any, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::symbol::{Symbol, terminals::TerminalKind};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct BitsTerminal {
    value: Vec<u8>,
    length: usize,
}

impl BitsTerminal {
    /// Create a new BitsTerminal.
    pub fn new(length: usize) -> Self {
        Self {
            value: vec![],
            length,
        }
    }
}

impl TerminalKind for BitsTerminal {
    fn display_terminal(&self) -> String {
        format!("Bits[{}]", self.length)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.length {
            Err(format!("Not enough data for Bits[{}]", self.length))
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
        state.write(b"BitsTerminal");
        state.write(&self.value);
    }
}

/// Create a Binary Bytes Terminal.
pub fn t_bytes(size: usize) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(BitsTerminal::new(size)),
    }
}
