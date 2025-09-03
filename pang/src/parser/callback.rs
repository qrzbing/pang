//! Callback can be used for Expansion to modify the parsed result.

use crate::symbol::DecodeError;

const SIZE_OF_USIZE: usize = size_of::<usize>();

/// Convert a little-endian byte array to a `usize`.
///
/// TODO: Support u128.
pub fn little_endian_bytes_to_usize(bytes: &[u8]) -> Result<usize, DecodeError> {
    // FIXME: This is a hack to support u32 only.
    if bytes.len() > 8 {
        return Err(DecodeError::Invalid(
            "Input slice cannot be larger than 4 bytes for this conversion",
        ));
    }

    if bytes.len() > SIZE_OF_USIZE {
        return Err(DecodeError::Invalid(
            "Terminal value is too large to fit in usize",
        ));
    }

    let mut bytes_array = [0u8; 8];
    bytes_array[0..bytes.len()].copy_from_slice(bytes);

    let value_u32 = u64::from_le_bytes(bytes_array);
    Ok(value_u32 as usize)
}

/// Convert a big-endian byte array to a `usize`.
///
/// TODO: Support u64 and u128.
pub fn big_endian_bytes_to_usize(bytes: &[u8]) -> Result<usize, DecodeError> {
    // FIXME: This is a hack to support u32 only.
    if bytes.len() > 4 {
        return Err(DecodeError::Invalid(
            "Input slice cannot be larger than 4 bytes for this conversion",
        ));
    }
    if bytes.len() > SIZE_OF_USIZE {
        return Err(DecodeError::Invalid(
            "Terminal value is too large to fit in usize",
        ));
    }

    let mut bytes_array = [0u8; 4];

    let start_pos = 4 - bytes.len();
    bytes_array[start_pos..].copy_from_slice(bytes);

    let value_u32 = u32::from_be_bytes(bytes_array);
    Ok(value_u32 as usize)
}
