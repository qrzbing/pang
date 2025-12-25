//! N or More Non Terminal

use std::{any::Any, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

#[allow(deprecated)]
use crate::{NonTerminalKind, Symbol};

/// NOrMore NonTerminal.
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

#[cfg(test)]
mod tests {
    use crate::nt;

    use super::*;

    #[test]
    fn test_n_or_more_non_terminal() {
        let nt_sym = nt("line");
        let nt_nom_sym = nt_nom("line", 2);
        assert_ne!(nt_sym, nt_nom_sym);
    }
}
