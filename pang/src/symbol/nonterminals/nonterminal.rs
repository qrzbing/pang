//! Simple Non-Terminal

use std::{any::Any, hash::Hasher, sync::Arc};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    DecodeError, DecodeResult, DerivationTree, Grammar, NonTerminalKind, ParseState, Symbol,
    new_node,
};

/// Bytes terminal.
/// TODO: support big/little-endian.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct NonTerminal {
    label: String,
}

impl NonTerminal {
    /// Create a new Non Terminal
    pub fn new(name: &str) -> Self {
        Self {
            label: name.to_string(),
        }
    }
}

#[typetag::serde]
impl NonTerminalKind for NonTerminal {
    fn display(&self) -> String {
        format!("<{}>", self.label)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn eq_dyn(&self, other: &dyn NonTerminalKind) -> bool {
        if let Some(other_val) = other.as_any().downcast_ref::<Self>() {
            self == other_val
        } else {
            false
        }
    }

    fn hash_dyn(&self, state: &mut dyn Hasher) {
        state.write(b"NonTerminal");
        state.write(&self.label.as_bytes());
    }

    fn parse<'a>(
        &self,
        state: &mut ParseState,
        input: &'a [u8],
        grammar: &'a Grammar,
    ) -> DecodeResult<'a, Arc<DerivationTree>> {
        let expansions = grammar.get(&self.label).ok_or(DecodeError::Invalid(
            "Non-terminal not found in grammar".into(),
        ))?;

        debug!(
            "    NT '{}': Found {} expansion(s)",
            self.label,
            expansions.len()
        );

        for (i, expansion) in expansions.iter().enumerate() {
            debug!("    NT '{}': Trying expansion #{}", self.label, i);

            let mut temp_state = state.clone();

            match expansion.parse(&mut temp_state, input, grammar) {
                Ok((remaining_input, children)) => {
                    *state = temp_state;
                    let node = new_node(nt(&self.label), Some(children));
                    state.context.insert(self.label.clone(), node.clone());

                    debug!(
                        "<-- SYMBOL PARSE SUCCESS (NT '{}'), remaining_len: {}",
                        self.label,
                        remaining_input.len()
                    );

                    return Ok((remaining_input, node));
                }
                Err(e) => {
                    debug!(
                        "    NT '{}': Expansion #{} FAILED. Reason: {:?}",
                        self.label, i, e
                    );
                }
            }
        }
        debug!(
            "<-- SYMBOL PARSE FAILED (NT '{}'): No expansion matched.",
            self.label
        );
        Err(DecodeError::Invalid(
            format!("No expansion matched for NT '{}'", self.label).into(),
        ))
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// Create a NonTerminal.
pub fn nt(label: &str) -> Symbol {
    Symbol::NonTerminal {
        kind: Arc::new(NonTerminal::new(label)),
    }
}
