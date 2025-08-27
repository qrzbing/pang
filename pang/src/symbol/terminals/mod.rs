//! # Terminal
//!
//! Terminal can not expand, but it can *genrate* a new value, or *parse* an input to a Terminal.
//!
//! This module contains some exmaples of terminals.

use std::{any::Any, collections::BTreeMap, fmt::Debug, hash::Hasher, sync::Arc};

use libafl_bolts::rands::Rand;

use crate::{
    symbol::{DecodeResult, SharedState, traits::HasLength},
    tree::DerivationTree,
};

///
pub mod ber_length;
///
pub mod bits;
///
pub mod bytes;
///
pub mod dynamic;
///
pub mod length_is;
///
pub mod literal;

///
pub trait DynRand {
    ///
    fn next(&mut self) -> u64;
    ///
    fn fill_bytes(&mut self, dest: &mut [u8]);
    ///
    fn below_or_zero(&mut self, n: usize) -> usize;
    ///
    fn between(&mut self, lower_bound_incl: usize, upper_bound_incl: usize) -> usize;
}

///
impl<R: Rand> DynRand for R {
    fn next(&mut self) -> u64 {
        Rand::next(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut left = dest;
        while left.len() >= 8 {
            let (l, r) = left.split_at_mut(8);
            left = r;
            let chunk: [u8; 8] = self.next().to_le_bytes();
            l.copy_from_slice(&chunk);
        }
        let n = left.len();
        if n > 0 {
            let chunk: [u8; 8] = self.next().to_le_bytes();
            left.copy_from_slice(&chunk[..n]);
        }
    }

    fn below_or_zero(&mut self, n: usize) -> usize {
        Rand::below_or_zero(self, n)
    }

    fn between(&mut self, lower_bound_incl: usize, upper_bound_incl: usize) -> usize {
        Rand::between(self, lower_bound_incl, upper_bound_incl)
    }
}

///
#[typetag::serde(tag = "type")]
pub trait TerminalKind: Debug + Send + Sync {
    /// Display the terminal in a human-readable format.
    fn display_terminal(&self) -> String;

    ///
    fn encode(&self) -> Result<Vec<u8>, String>;

    ///
    fn generate(&self, rng: &mut dyn DynRand) -> Arc<DerivationTree>;

    /// Returns a `&dyn Any` reference to itself for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Dynamically compare two [`TerminalKind`] trait objects
    fn eq_dyn(&self, other: &dyn TerminalKind) -> bool;

    ///
    fn hash_dyn(&self, state: &mut dyn Hasher);

    /// Parse the terminal from input.
    fn parse<'a>(
        &self,
        input: &'a [u8],
        state: &SharedState,
        context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>>;

    /// Convert a Terminal to a [`HasLength`] trait object.
    /// By default, returns `None`.
    fn as_has_length(&self) -> Option<&dyn HasLength> {
        None
    }
}
