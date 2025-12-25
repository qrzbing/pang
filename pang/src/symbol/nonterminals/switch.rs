//! Switch Non Terminal

use std::{any::Any, collections::HashMap, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

#[allow(deprecated)]
use crate::{EnumMapping, NonTerminalKind, Symbol};

/// Switch NonTerminal.
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
pub struct SwitchNonTerminal {
    /// Dependent Variable
    dep_name: String,
    /// Default Label
    default_label: Option<String>,
}

#[typetag::serde]
impl NonTerminalKind for SwitchNonTerminal {
    fn display(&self) -> String {
        format!("<{}>", self.dep_name)
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
        state.write(b"SwitchNonTerminal");
        state.write(&self.dep_name.as_bytes());
    }

    fn label(&self) -> &str {
        &self.dep_name
    }

    fn references(&self, enums: &HashMap<String, EnumMapping>) -> Vec<String> {
        let mut refs = Vec::new();

        // Add fallback label
        if let Some(label) = &self.default_label {
            refs.push(label.clone());
        }

        // Add Enums
        if let Some(mapping) = enums.get(&self.dep_name) {
            match mapping {
                EnumMapping::Int(map) => {
                    for target_label in map.values() {
                        refs.push(target_label.clone());
                    }
                }
            }
        }

        refs
    }
}

/// Create a Switch NonTerminal
pub fn nt_switch(dep_name: &str, default_label: Option<&str>) -> Symbol {
    Symbol::NonTerminal {
        kind: Arc::new(SwitchNonTerminal {
            dep_name: dep_name.to_string(),
            default_label: default_label.map(|s| s.to_string()),
        }),
    }
}
