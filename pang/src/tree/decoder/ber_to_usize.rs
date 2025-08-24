use crate::tree::decoder::{DecodeError, DecodeResult};

/// Decode a BER-encoded length to a `usize`.
///
/// # Example
///
/// ```
/// use pang::{
///     grammar::{nt, t_dyn},
///     tree::{decoder::ber_to_usize, new_node},
/// };
/// let tree = new_node(
///     nt("asn1-tlv-len"),
///     Some(vec![new_node(t_dyn(), None, Some(vec![0x00]))]),
///     None,
/// );
/// let asn1_tlv_len = tree.decode(ber_to_usize).unwrap();
/// assert_eq!(asn1_tlv_len, 0);
/// let tree = new_node(
///     nt("asn1-tlv-len"),
///     Some(vec![new_node(t_dyn(), None, Some(vec![0x7f]))]),
///     None,
/// );
/// let asn1_tlv_len = tree.decode(ber_to_usize).unwrap();
/// assert_eq!(asn1_tlv_len, 0x7f);
/// let tree = new_node(
///     nt("asn1-tlv-len"),
///     Some(vec![new_node(t_dyn(), None, Some(vec![0x82, 0x01, 0x10]))]),
///     None,
/// );
/// let asn1_tlv_len = tree.decode(ber_to_usize).unwrap();
/// assert_eq!(asn1_tlv_len, 272);  // 0x110
/// ```
pub fn ber_to_usize(input: &[u8]) -> DecodeResult<usize> {
    if input.is_empty() {
        return Err(DecodeError::Incomplete);
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
            return Err(DecodeError::InvalidData(
                "Invalid BER long form: number of length bytes cannot be zero",
            ));
        }

        // Check if the actual number of bytes matches the number advertised.
        // The total length of the slice should be 1 (for the initial byte) + num_len_bytes.
        if input.len() < 1 + num_len_bytes {
            return Err(DecodeError::Incomplete);
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
        return Err(DecodeError::Incomplete);
    }

    let remaining = &input[field_len..];
    Ok((remaining, value_len))
}
