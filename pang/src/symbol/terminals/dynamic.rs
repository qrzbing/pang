//! # DynamicTerminal

use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    symbol::{
        DecodeResult, SharedState, Symbol,
        terminals::{DynRand, TerminalKind},
    },
    tree::{DerivationTree, new_node},
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
        format!("Dyn[{}]", self.length)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.length {
            Err(format!("Not enough data for Bytes[{}]", self.length))
        } else {
            Ok(self.value[..self.length].to_vec())
        }
    }

    fn generate(&self, rng: &mut dyn DynRand) -> Arc<DerivationTree> {
        let size = rng.between(8, 16);
        let mut generate_bytes = vec![0u8; size];
        rng.fill_bytes(&mut generate_bytes);
        new_node(
            Symbol::Terminal {
                kind: Arc::new(DynamicTerminal::from_bytes(&generate_bytes)),
            },
            Some(vec![]),
        )
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
        kind: Arc::new(DynamicTerminal::new()),
    }
}

/// Consume all the input and return DynamicTerminal.
pub fn t_dyn_value(value: &[u8]) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(DynamicTerminal::from_bytes(value)),
    }
}
