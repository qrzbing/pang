//! N or More Non Terminal

use std::{any::Any, collections::BTreeMap, hash::Hasher, sync::Arc};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{
    DecodeResult, DerivationTree, Grammar, NonTerminalKind, Symbol, new_node, nt,
    terminals::DynRand,
};

/// Bytes terminal.
/// TODO: support big/little-endian.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct NOrMoreNonTerminal {
    label: String,
    minimum_repeat_time: u32,
}

impl NOrMoreNonTerminal {
    /// Create a new N or More Non Terminal.
    pub fn new(name: &str, minimum_repeat_time: u32) -> Self {
        Self {
            label: name.to_string(),
            minimum_repeat_time,
        }
    }
}

#[typetag::serde]
impl NonTerminalKind for NOrMoreNonTerminal {
    fn display(&self) -> String {
        format!("{}*(<{}>)", self.minimum_repeat_time, self.label)
    }

    fn encode(&self) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn generate(
        &self,
        _grammar: &Grammar,
        _rng: &mut dyn DynRand,
    ) -> Result<Arc<DerivationTree>, String> {
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
        state.write(b"NOrMoreNonTerminal");
        state.write(&self.label.as_bytes());
    }

    fn parse<'a>(
        &self,
        input: &'a [u8],
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<DerivationTree>> {
        let mut children = Vec::new();
        let mut current_input = input;
        let inner_symbol = nt(&self.label);

        // Must match ar least `minimum_repeat_time` times
        for _ in 0..self.minimum_repeat_time {
            match inner_symbol.parse(current_input, grammar, &mut context.clone()) {
                Ok((next_input, child_node)) => {
                    current_input = next_input;
                    children.push(child_node);
                }
                Err(e) => {
                    debug!(
                        "<-- SYMBOL PARSE FAILED ({} Or More '{}'): Did not match even once.",
                        self.minimum_repeat_time, self.label
                    );
                    return Err(e);
                }
            }
        }

        loop {
            match inner_symbol.parse(current_input, grammar, &mut context.clone()) {
                Ok((next_input, child_node)) => {
                    // Match 0 time, do not consume any input.
                    if next_input.len() == current_input.len() {
                        break;
                    }
                    current_input = next_input;
                    children.push(child_node);
                }
                Err(_) => {
                    break;
                }
            }
        }

        let node = new_node(nt_nom(&self.label, self.minimum_repeat_time), Some(children));
        debug!(
            "<-- SYMBOL PARSE SUCCESS (NOrMore '{}'), matched {} times, remaining_len: {}",
            self.label,
            node.children.as_ref().map_or(0, |c| c.len()),
            current_input.len()
        );
        Ok((current_input, node))
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// Create a NonTerminal.
pub fn nt_nom(label: &str, minimum_repeat_time: u32) -> Symbol {
    Symbol::NonTerminal {
        kind: Arc::new(NOrMoreNonTerminal::new(label, minimum_repeat_time)),
    }
}
