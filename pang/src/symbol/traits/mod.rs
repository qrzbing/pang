//! Useful traits for symbols.

use std::sync::Arc;

use crate::symbol::terminals::TerminalKind;

/// HasLength can make symbol represent as a length value.
pub trait HasLength {
    /// Returns the length value represented by this terminal.
    fn as_length(&self) -> Option<usize>;

    /// Create a [`TerminalKind`] from a length value.
    fn from_length(&self, len: usize) -> Arc<dyn TerminalKind>;
}
