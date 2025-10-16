//! # BerLengthTerminal
//!
//! BerLengthTerminal can represent a BER-encoded length field.
//!
//! ## Example
//!
//! ```
//! use std::collections::BTreeMap;
//!
//! use pang::{BerLengthTerminal, Grammar, Symbol, TerminalKind, t_ber};
//!
//! let ber_symbol = t_ber();
//! assert_eq!(ber_symbol.to_string(), "0");
//!
//! if let Symbol::Terminal {
//!     label: _,
//!     kind: ber_kind,
//! } = ber_symbol
//! {
//!     let ber_len = ber_kind
//!         .as_any()
//!         .downcast_ref::<BerLengthTerminal>()
//!         .unwrap()
//!         .value();
//!     assert_eq!(ber_len, 0);
//! }
//!
//! let ber_terminal = BerLengthTerminal::from_usize(255);
//! assert_eq!(ber_terminal.encode(), Ok(vec![0x81, 0xff]));
//!
//! let ber_symbol = t_ber();
//! let grammar = Grammar::new();
//! let input = &[0x82, 0xff, 0xff];
//! let (remaining_input, ber_kind) = ber_symbol
//!     .parse(input, &grammar, &mut BTreeMap::new())
//!     .unwrap();
//! assert_eq!(remaining_input, &[] as &[u8]);
//! let ber_len = ber_kind
//!     .first_terminal_kind()
//!     .unwrap()
//!     .as_any()
//!     .downcast_ref::<BerLengthTerminal>()
//!     .unwrap()
//!     .value();
//! assert_eq!(ber_len, 65535);
//! ```

use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    symbol::{DecodeError, DecodeResult, SharedState, Symbol, terminals::TerminalKind},
    tree::DerivationTree,
};

/// Convert a usize to a BER-encoded length field.
fn usize_to_ber_bytes(len: usize) -> Vec<u8> {
    if len < 128 {
        // Less than 0x80
        vec![len as u8]
    } else {
        // More than 0x80
        let len_bytes = len.to_be_bytes();
        // Find the first non-zero byte (if any)
        let first_byte_idx = len_bytes
            .iter()
            .position(|&b| b != 0)
            .unwrap_or(len_bytes.len());
        let num_len_bytes = len_bytes.len() - first_byte_idx;

        let mut result = Vec::with_capacity(1 + num_len_bytes);
        // Write first byte (with MSB set)
        result.push(0x80 | num_len_bytes as u8);
        // Write length itself (big-endian)
        result.extend_from_slice(&len_bytes[first_byte_idx..]);
        result
    }
}

/// Decode a BER-encoded length to a `usize`.
fn ber_to_usize(input: &[u8]) -> DecodeResult<'_, usize> {
    if input.is_empty() {
        return Err(DecodeError::Incomplete("Input is empty"));
    }

    let first_byte = input[0];
    let (field_len, value_len) = if (first_byte & 0x80) == 0 {
        // In short form, the length is the byte itself.
        // The field must be exactly one byte long.
        (1, first_byte as usize)
    } else {
        // // Long form (MSB is 1)
        let num_len_bytes = (first_byte & 0x7F) as usize;
        if num_len_bytes == 0 {
            return Err(DecodeError::Invalid(
                "Invalid BER long form: number of length bytes cannot be zero".into(),
            ));
        }

        // Check if the actual number of bytes matches the number advertised.
        // The total length of the slice should be 1 (for the initial byte) + num_len_bytes.
        if input.len() < 1 + num_len_bytes {
            return Err(DecodeError::Incomplete(
                "Input length too short for BER long form",
            ));
        }

        let len_bytes = &input[1..1 + num_len_bytes];
        let mut length: usize = 0;
        for &byte in len_bytes {
            // Manual big-endian conversion
            length = (length << 8) + (byte as usize);
        }
        (1 + num_len_bytes, length)
    };

    if input.len() < field_len {
        return Err(DecodeError::Incomplete(
            "Input length too short for BER field",
        ));
    }

    let remaining = &input[field_len..];
    Ok((remaining, value_len))
}

/// BER-encoded length terminal.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct BerLengthTerminal {
    value: usize,
}

impl BerLengthTerminal {
    /// Create a new BerLengthTerminal.
    pub fn new() -> Self {
        Self { value: 0 }
    }

    /// Create a new BerLengthTerminal from a `usize`.
    pub fn from_usize(value: usize) -> Self {
        Self { value }
    }

    /// Return the value of the BerLengthTerminal.
    pub fn value(&self) -> usize {
        self.value
    }
}

#[typetag::serde]
impl TerminalKind for BerLengthTerminal {
    fn display_terminal(&self) -> String {
        format!("{}", self.value)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        Ok(usize_to_ber_bytes(self.value))
    }

    fn parse<'a>(
        &self,
        input: &'a [u8],
        _state: &SharedState,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        let (remaining_input, value) = ber_to_usize(input)?;
        Ok((
            remaining_input,
            Arc::new(BerLengthTerminal::from_usize(value)),
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
        state.write(b"BerLengthTerminal");
        state.write(&self.value.to_be_bytes());
    }
}

/// Create a BER Length Terminal.
pub fn t_ber() -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(BerLengthTerminal::new()),
    }
}

/// Create a BER Length Terminal.
pub fn tl_ber(label: &str) -> Symbol {
    Symbol::Terminal {
        label: Some(label.into()),
        kind: Arc::new(BerLengthTerminal::new()),
    }
}

/// Create a BER Length Terminal with value.
pub fn t_ber_val(value: usize) -> Symbol {
    Symbol::Terminal {
        label: None,
        kind: Arc::new(BerLengthTerminal::from_usize(value)),
    }
}

/// Create a BER Length Terminal with value and label.
pub fn tl_ber_val(label: &str, value: usize) -> Symbol {
    Symbol::Terminal {
        label: Some(label.into()),
        kind: Arc::new(BerLengthTerminal::from_usize(value)),
    }
}
