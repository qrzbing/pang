//! Decoder for Derivation Tree

use std::fmt;

mod ber_to_usize;
pub use ber_to_usize::ber_to_usize;

/// Decode error types.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    /// Incomplete data
    Incomplete,
    /// Invalid data format
    InvalidData(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Incomplete => write!(f, "Incomplete data"),
            DecodeError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for DecodeError {}

/// DecodeResult type alias
///
/// Returns a tuple of the decoded value and the remaining data.
pub type DecodeResult<'a, T> = Result<(T, &'a [u8]), DecodeError>;

/// Custom decoder function type alias
pub type CustomDecoderFn<'a, T> = fn(&'a [u8]) -> DecodeResult<'a, T>;

/// NodeValue
#[derive(Debug)]
pub struct NodeValue<'a> {
    bytes: &'a [u8],
}

impl<'a> NodeValue<'a> {
    /// Create a NodeValue
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// Decode the bytes using a custom decoder function
    pub fn decode<T>(&self, decoder: CustomDecoderFn<'a, T>) -> Result<T, DecodeError> {
        match decoder(self.bytes) {
            Ok((value, remaining)) => {
                if !remaining.is_empty() {
                    Err(DecodeError::InvalidData(
                        "Expected end of input, but data remains",
                    ))
                } else {
                    Ok(value)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Decode the bytes using a custom decoder function, returning the remaining data
    pub fn decode_partial<T>(
        &self,
        decoder: CustomDecoderFn<'a, T>,
    ) -> Result<(T, &'a [u8]), DecodeError> {
        decoder(self.bytes)
    }

    /// Visit raw bytes
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}
