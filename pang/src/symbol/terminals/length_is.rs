use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

use crate::{
    symbol::{DecodeResult, SharedState, Symbol, terminals::TerminalKind},
    tree::DerivationTree,
};

///
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct LengthIsTerminal {
    value: Vec<u8>,
    length: usize,

    symbol_name: String,
}

impl LengthIsTerminal {
    /// Create a new LengthIsTerminal.
    pub fn new(symbol_name: &str) -> Self {
        Self {
            value: vec![],
            length: 0,

            symbol_name: symbol_name.to_string(),
        }
    }
}

#[typetag::serde]
impl TerminalKind for LengthIsTerminal {
    fn display_terminal(&self) -> String {
        format!("Bytes[{}]", self.length)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        if self.value.len() < self.length {
            Err(format!("Not enough data for Bytes[{}]", self.length))
        } else {
            Ok(self.value[..self.length].to_vec())
        }
    }

    fn generate(&self, _rng: &mut ThreadRng) -> Arc<DerivationTree> {
        todo!("Implement it later.")
    }

    fn parse<'a>(
        &self,
        _input: &'a [u8],
        _state: &SharedState,
        _context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        todo!("Implement it later.")
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
        state.write(b"LengthIsTerminal");
        state.write(&self.value);
    }
}

///
pub fn t_length_is(symbol_name: &str) -> Symbol {
    Symbol::Terminal {
        kind: Arc::new(LengthIsTerminal::new(symbol_name)),
    }
}
