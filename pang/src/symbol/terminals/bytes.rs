//! # BytesTerminal

use std::{any::Any, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    display_u8,
    symbol::{DecodeError, DecodeResult, SharedState, Symbol, terminals::TerminalKind},
};

/// Bytes terminal.
/// TODO: support big/little-endian.
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

    /// Create a new BytesTerminal from a byte slice.
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
        if self.value.is_empty() {
            format!("Bytes[{}]", self.size)
        } else {
            format!("Bytes[{}]: {}", self.size, display_u8(&self.value))
        }
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.size {
            Err(format!("Not enough data for Bytes[{}]", self.size))
        } else {
            Ok(self.value[..self.size].to_vec())
        }
    }

    fn parse<'a>(
        &self,
        input: &'a [u8],
        _state: &SharedState,
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
}

/// Create a Binary Bytes Terminal without label.
pub fn t_bytes(size: usize) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(BytesTerminal::new_from_len(size)),
    }
}

/// Create a Binary Bytes Terminal with label.
pub fn tl_bytes(label: &str, size: usize) -> Symbol {
    Symbol::Terminal {
        label: Some(label.to_string()),
        kind: Arc::new(BytesTerminal::new_from_len(size)),
    }
}

/// Create a Binary Bytes Terminal from a byte slice without label.
pub fn t_bytes_val(val: &[u8]) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(BytesTerminal::new_from_val(val)),
    }
}

/// Create a Binary Bytes Terminal from a byte slice with label.
pub fn tl_bytes_val(label: &str, val: &[u8]) -> Symbol {
    Symbol::Terminal {
        label: Some(label.to_string()),
        kind: Arc::new(BytesTerminal::new_from_val(val)),
    }
}
