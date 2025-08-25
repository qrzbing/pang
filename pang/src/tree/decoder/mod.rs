//! Decoder for Derivation Tree

use crate::symbol::{DecodeError, DecodeResult};

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
            Ok((remaining, value)) => {
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
    ) -> Result<(&'a [u8], T), DecodeError> {
        decoder(self.bytes)
    }

    /// Visit raw bytes
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}
