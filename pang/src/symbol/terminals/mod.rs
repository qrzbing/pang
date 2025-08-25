// !
use std::{any::Any, fmt::Debug, hash::Hasher, sync::Arc};

use rand::rngs::ThreadRng;

use crate::tree::DerivationTree;

///
pub mod ber_length;
///
pub mod bits;
///
pub mod bytes;
///
pub mod dynamic;
///
pub mod literal;

///
#[typetag::serde(tag = "type")]
pub trait TerminalKind: Debug + Send + Sync {
    /// Display the terminal in a human-readable format.
    fn display_terminal(&self) -> String;
    ///
    fn encode(&self) -> Result<Vec<u8>, String>;
    ///
    fn generate(&self, rng: &mut ThreadRng) -> Arc<DerivationTree>;
    /// Returns a `&dyn Any` reference to itself for downcasting
    fn as_any(&self) -> &dyn Any;
    /// Dynamically compare two [`TerminalKind`] trait objects
    fn eq_dyn(&self, other: &dyn TerminalKind) -> bool;
    ///
    fn hash_dyn(&self, state: &mut dyn Hasher);
}
