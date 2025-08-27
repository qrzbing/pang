//! Useful traits for symbols.

/// HasLength can make symbol represent as a length value.
pub trait HasLength {
    /// Returns the length value represented by this terminal.
    fn as_length(&self) -> Option<usize>;
}
