use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use rand::{RngCore, rngs::ThreadRng};
use serde::{Deserialize, Serialize};

use crate::{
    symbol::{
        DecodeError, DecodeResult, SharedState, Symbol, terminals::TerminalKind, traits::HasLength,
    },
    tree::{DerivationTree, new_node},
};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct BytesTerminal {
    value: Vec<u8>,
    size: usize,
}

impl BytesTerminal {
    /// Create a new BytesTerminal.
    pub fn new_from_len(size: usize) -> Self {
        Self {
            value: vec![],
            size,
        }
    }

    /// Create a new BytesTerminal.
    pub fn new_from_val(value: &[u8]) -> Self {
        Self {
            value: value.to_vec(),
            size: value.len(),
        }
    }
}

#[typetag::serde]
impl TerminalKind for BytesTerminal {
    fn display_terminal(&self) -> String {
        format!("Bytes[{}]", self.size)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.size {
            Err(format!("Not enough data for Bytes[{}]", self.size))
        } else {
            Ok(self.value[..self.size].to_vec())
        }
    }

    fn generate(&self, rng: &mut ThreadRng) -> Arc<DerivationTree> {
        let mut generate_bytes = vec![0u8; self.size];
        rng.fill_bytes(&mut generate_bytes);
        new_node(t_bytes_val(&generate_bytes), Some(vec![]))
    }

    fn parse<'a>(
        &self,
        input: &'a [u8],
        _state: &SharedState,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        assert_ne!(self.size, 0);
        if input.len() < self.size {
            return Err(DecodeError::Incomplete(
                "Input length is less than expected size",
            ));
        }
        let (consumed_slice, remaining_input) = input.split_at(self.size);
        Ok((
            remaining_input,
            Arc::new(BytesTerminal::new_from_val(consumed_slice)),
        ))
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

    fn as_has_length(&self) -> Option<&dyn HasLength> {
        Some(self)
    }
}

impl HasLength for BytesTerminal {
    fn as_length(&self) -> Option<usize> {
        if self.value.is_empty() {
            return None;
        }
        let mut buf = [0u8; size_of::<usize>()];
        let buf_len = buf.len();
        let len = self.value.len().min(buf_len);
        buf[buf_len - len..].copy_from_slice(&self.value[..len]);
        Some(usize::from_be_bytes(buf))
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
