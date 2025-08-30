//! Callback can be used for Expansion to modify the parsed result.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{symbol::DecodeError, tree::DerivationTree};

const SIZE_OF_USIZE: usize = size_of::<usize>();

/// A helper function to easily extract a `usize` value from a parsed symbol in the context.
///
/// This is intended to be used within callback functions. It retrieves a symbol by its label,
/// finds the first terminal, and converts its value to a `usize`.
pub fn get_symbol_val(
    context: &BTreeMap<String, Arc<DerivationTree>>,
    label: &str,
) -> Result<usize, DecodeError> {
    let node = context.get(label).ok_or(DecodeError::Invalid(
        "Symbol not found in context for length calculation",
    ))?;

    let terminal_kind = node.first_terminal_kind().ok_or(DecodeError::Invalid(
        "No terminal found in symbol for length calculation",
    ))?;

    // Prefer using the HasLength trait if it's implemented.
    if let Some(has_length) = terminal_kind.as_has_length() {
        return has_length.as_length().ok_or(DecodeError::Invalid(
            "Could not determine length from terminal that implements HasLength",
        ));
    }

    // Fallback: encode the terminal to bytes and interpret as a big-endian integer.
    let bytes = terminal_kind
        .encode()
        .map_err(|e| DecodeError::Invalid(Box::leak(e.into_boxed_str())))?;
    if bytes.len() > SIZE_OF_USIZE {
        return Err(DecodeError::Invalid(
            "Terminal value is too large to fit in usize",
        ));
    }

    let mut buf = [0u8; SIZE_OF_USIZE];
    buf[(SIZE_OF_USIZE - bytes.len())..].copy_from_slice(&bytes);
    Ok(usize::from_be_bytes(buf))
}
