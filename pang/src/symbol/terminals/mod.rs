//! # Terminal
//!
//! Terminal can not expand, but it can *genrate* a new value, or *parse* an input to a Terminal.
//!
//! This module contains some exmaples of terminals.

use std::{any::Any, fmt::Debug, hash::Hasher};

pub mod ber_length;
pub use ber_length::*;
pub mod bits;
pub use bits::*;
pub mod bytes;
pub use bytes::*;
pub mod dynamic;
pub use dynamic::*;
pub mod literal;
pub use literal::*;
use serde::{Deserialize, Serialize};

/// TerminalKind can describe a terminal symbol.
#[typetag::serde(tag = "type")]
pub trait TerminalKind: Debug + Send + Sync {
    /// Display the terminal in a human-readable format.
    fn display_terminal(&self) -> String;

    /// Encode the terminal to a byte array.
    fn encode(&self) -> Result<Vec<u8>, String>;

    /// Returns a `&dyn Any` reference to itself for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Dynamically compare two [`TerminalKind`] trait objects
    fn eq_dyn(&self, other: &dyn TerminalKind) -> bool;

    /// Hash the terminal using the given Hasher.
    fn hash_dyn(&self, state: &mut dyn Hasher);
}

/// NopTerminal
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct NopTerminal;

impl NopTerminal {
    /// Create a new NopTerminal.
    pub fn new() -> Self {
        NopTerminal {}
    }
}

#[typetag::serde]
impl TerminalKind for NopTerminal {
    fn display_terminal(&self) -> String {
        "None".into()
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        Ok("".into())
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
        state.write(b"NopTerminal");
    }
}
