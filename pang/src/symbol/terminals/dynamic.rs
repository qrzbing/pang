use std::{any::Any, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::symbol::{Symbol, terminals::TerminalKind};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct DynamicTerminal {
    value: Vec<u8>,
    length: usize,
}

impl DynamicTerminal {
    /// Create a new DynamicTerminal.
    pub fn new() -> Self {
        Self {
            value: vec![],
            length: 0,
        }
    }
}

impl TerminalKind for DynamicTerminal {
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
        state.write(b"DynamicTerminal");
        state.write(&self.value);
    }
}

///
pub fn t_dyn() -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(DynamicTerminal::new()),
    }
}
