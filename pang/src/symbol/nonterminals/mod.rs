//! # Non Terminal
//!
//! Non Terminal can be expanded to a set of Terminals.
//!
//! This module contains some exmaples of non terminals.

use std::{any::Any, collections::BTreeMap, fmt::Debug, hash::Hasher, sync::Arc};

use crate::{DecodeResult, DerivationTree, Grammar, terminals::DynRand};

pub mod nonterminal;
pub use nonterminal::nt;
pub mod normore;
pub use normore::nomt;

/// NonTerminalKind can describe a non-terminal symbol.
#[typetag::serde(tag = "type")]
pub trait NonTerminalKind: Debug + Send + Sync {
    /// Display the non-terminal in a human-readable format.
    fn display(&self) -> String;

    /// Encode the non-terminal to a byte array.
    fn encode(&self) -> Result<Vec<u8>, String>;

    /// Generate a new value for the non-terminal.
    fn generate(
        &self,
        grammar: &Grammar,
        rng: &mut dyn DynRand,
    ) -> Result<Arc<DerivationTree>, String>;

    /// Returns a `&dyn Any` reference to itself for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Dynamically compare two [`NonTerminalKind`] trait objects
    fn eq_dyn(&self, other: &dyn NonTerminalKind) -> bool;

    /// Hash the non-terminal using the given Hasher.
    fn hash_dyn(&self, state: &mut dyn Hasher);

    /// Parse the terminal from input.
    fn parse<'a>(
        &self,
        input: &'a [u8],
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<DerivationTree>>;

    /// Return the name of a non-terminal
    fn label(&self) -> &str;
}
