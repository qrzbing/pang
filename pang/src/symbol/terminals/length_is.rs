use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use log::debug;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};

use crate::{
    symbol::{DecodeError, DecodeResult, SharedState, Symbol, terminals::TerminalKind},
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

    /// Create a new LengthIsTerminal from bytes.
    pub fn from_bytes(bytes: &[u8], symbol_name: &str) -> Self {
        Self {
            value: bytes.to_vec(),
            length: bytes.len(),

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
        input: &'a [u8],
        _state: &SharedState,
        context: &BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<dyn TerminalKind>> {
        // Find length node in context
        let length_node = context
            .get(&self.symbol_name)
            .ok_or_else(|| DecodeError::Invalid("Length field node not found in context"))?;

        debug!("Length field node:\n{}", length_node);

        // Get TerminalKind from length node
        let length_terminal_kind =
            length_node
                .first_terminal_kind()
                .ok_or(DecodeError::Invalid(
                    "No terminal found under the length field node",
                ))?;

        // Convert length node to usize
        let dynamic_size = length_terminal_kind
            .as_has_length()
            .ok_or(DecodeError::Invalid(
                "Length field terminal does not implement HasLength",
            ))?
            .as_length()
            .ok_or(DecodeError::Invalid(
                "Length value could not be determined from terminal",
            ))?;

        debug!("Get length from context: {}", dynamic_size);

        if input.len() < dynamic_size {
            return Err(DecodeError::Incomplete(
                "Not enough data for LengthIsTerminal",
            ));
        }
        let (consumed_slice, remaining_input) = input.split_at(dynamic_size);

        Ok((
            remaining_input,
            Arc::new(LengthIsTerminal::from_bytes(
                consumed_slice,
                &self.symbol_name,
            )),
        ))
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
