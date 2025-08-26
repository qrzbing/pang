use std::{
    any::Any,
    collections::BTreeMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

use crate::{
    symbol::{DecodeResult, SharedState, Symbol, terminals::TerminalKind},
    tree::{DerivationTree, new_node},
};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct LiteralTerminal {
    ///
    pub value: String,
}

impl LiteralTerminal {
    /// Create a new LiteralTerminal.
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[typetag::serde]
impl TerminalKind for LiteralTerminal {
    fn display_terminal(&self) -> String {
        format!("{}", self.value)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        Ok(self.value.as_bytes().to_vec())
    }

    fn generate(&self, _rng: &mut ThreadRng) -> Arc<DerivationTree> {
        new_node(t(&self.value), Some(vec![]))
    }

    fn parse<'a>(
        &self,
        _input: &'a [u8],
        _state: &SharedState,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        todo!("As LiteralTerminal is used to parse strings, now we don't implement it.")
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
        state.write(b"LiteralTerminal");
        state.write(self.value.as_bytes());
    }
}

/// Create a Literal Terminal.
pub fn t(value: &str) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(LiteralTerminal::new(value.to_string())),
    }
}
