//! # DynamicTerminal

use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    display_u8,
    symbol::{
        DecodeResult, SharedState, Symbol,
        terminals::{DynRand, TerminalKind},
    },
    tree::DerivationTree,
};

/// DynamicTerminal.
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

    /// Create a new DynamicTerminal from bytes.
    pub fn from_bytes(inp: &[u8]) -> Self {
        let size = inp.len();
        Self {
            value: inp.to_vec(),
            length: size,
        }
    }
}

#[typetag::serde]
impl TerminalKind for DynamicTerminal {
    fn display_terminal(&self) -> String {
        format!("Dyn[{}]: {}", self.length, display_u8(&self.value))
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.length {
            Err(format!("Not enough data for Bytes[{}]", self.length))
        } else {
            Ok(self.value[..self.length].to_vec())
        }
    }

    fn generate(&self, rng: &mut dyn DynRand) -> Arc<dyn TerminalKind> {
        let size = rng.between(8, 16);
        let mut generate_bytes = vec![0u8; size];
        rng.fill_bytes(&mut generate_bytes);
        Arc::new(DynamicTerminal::from_bytes(&generate_bytes))
    }

    fn parse<'a>(
        &self,
        input: &'a [u8],
        _state: &SharedState,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        // In DynamicTerminal, we don't need to parse anything.
        // Consume all the input and return the terminal.
        Ok((b"", Arc::new(DynamicTerminal::from_bytes(input))))
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

/// Return a new DynamicTerminal.
pub fn t_dyn() -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(DynamicTerminal::new()),
    }
}

/// Return a new DynamicTerminal.
pub fn tl_dyn(label: &str) -> Symbol {
    Symbol::Terminal {
        label: Some(label.into()),
        kind: Arc::new(DynamicTerminal::new()),
    }
}

/// Consume all the input and return DynamicTerminal.
pub fn t_dyn_val(value: &[u8]) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(DynamicTerminal::from_bytes(value)),
    }
}

/// Consume all the input and return DynamicTerminal.
pub fn tl_dyn_val(label: &str, value: &[u8]) -> Symbol {
    Symbol::Terminal {
        label: Some(label.into()),
        kind: Arc::new(DynamicTerminal::from_bytes(value)),
    }
}
