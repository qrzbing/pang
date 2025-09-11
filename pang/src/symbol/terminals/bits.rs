//! BitsTerminal

use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    symbol::{
        DecodeResult, SharedState, Symbol,
        terminals::{DynRand, TerminalKind},
    },
    tree::DerivationTree,
};

/// Bits terminal, not fully implemented yet.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct BitsTerminal {
    value: Vec<u8>,
    size: usize,
}

impl BitsTerminal {
    /// Create a new BitsTerminal.
    pub fn new(size: usize) -> Self {
        Self {
            value: vec![],
            size,
        }
    }

    /// FIXME: Create a new BitsTerminal.
    pub fn new_from_val(value: &[u8]) -> Self {
        Self {
            value: value.to_vec(),
            size: value.len(),
        }
    }
}

#[typetag::serde]
impl TerminalKind for BitsTerminal {
    fn display_terminal(&self) -> String {
        format!("Bits[{}]", self.size)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.size {
            Err(format!("Not enough data for Bits[{}]", self.size))
        } else {
            Ok(self.value[..self.size].to_vec())
        }
    }

    fn generate(&self, rng: &mut dyn DynRand) -> Arc<dyn TerminalKind> {
        let mut generate_bytes = vec![0u8; self.size];
        rng.fill_bytes(&mut generate_bytes);
        // FIXME: Will not work as expected.
        Arc::new(BitsTerminal::new(self.size))
    }

    fn parse<'a>(
        &self,
        _input: &'a [u8],
        _state: &SharedState,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        todo!("Implement it later.")
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

/// Create a Binary Bits Terminal.
pub fn t_bits(size: usize) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(BitsTerminal::new(size)),
    }
}

/// Create a Binary Bits Terminal.
pub fn tl_bits(label: &str, size: usize) -> Symbol {
    Symbol::Terminal {
        label: Some(label.to_string()),
        kind: Arc::new(BitsTerminal::new(size)),
    }
}

/// FIXME: Create a Binary Bits Terminal.
pub fn t_bits_val(val: &[u8]) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(BitsTerminal::new_from_val(val)),
    }
}

/// FIXME: Create a Binary Bits Terminal.
pub fn tl_bits_val(label: &str, val: &[u8]) -> Symbol {
    Symbol::Terminal {
        label: Some(label.to_string()),
        kind: Arc::new(BitsTerminal::new_from_val(val)),
    }
}
