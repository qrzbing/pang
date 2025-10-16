//! Simple Non-Terminal

use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    DecodeError, DecodeResult, DerivationTree, Grammar, NonTerminalKind, Symbol, new_node,
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
        input: &'a [u8],
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
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

            let mut temp_context = context.clone();

            if let Ok((remaining_input, children)) =
                expansion.parse(input, grammar, &mut temp_context)
            {
                *context = temp_context;
                let node = new_node(nt(&self.label), Some(children));
                context.insert(self.label.clone(), node.clone());

                debug!(
                    "<-- SYMBOL PARSE SUCCESS (NT '{}'), remaining_len: {}",
                    self.label,
                    remaining_input.len()
                );

                return Ok((remaining_input, node));
            } else {
                debug!("    NT '{}': Expansion #{} FAILED.", self.label, i);
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
