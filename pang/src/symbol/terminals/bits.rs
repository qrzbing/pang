use std::{any::Any, hash::Hasher, sync::Arc};

use rand::{RngCore, rngs::ThreadRng};
use serde::{Deserialize, Serialize};

use crate::{
    symbol::{Symbol, terminals::TerminalKind},
    tree::{DerivationTree, new_node},
};

///
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

    fn generate(&self, rng: &mut ThreadRng) -> Arc<DerivationTree> {
        let mut generate_bytes = vec![0u8; self.size];
        rng.fill_bytes(&mut generate_bytes);
        new_node(t_bits(self.size), Some(vec![]))
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
        kind: Arc::new(BitsTerminal::new(size)),
    }
}

/// FIXME: Create a Binary Bits Terminal.
pub fn t_bis_val(val: &[u8]) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(BitsTerminal::new_from_val(val)),
    }
}
