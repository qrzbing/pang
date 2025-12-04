//! Switch Non Terminal

use std::{any::Any, collections::HashMap, hash::Hasher, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    DecodeError, DecodeResult, EnumMapping, NonTerminalKind, ParseState, Symbol,
    callback::big_endian_bytes_to_usize, nt,
};

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

    fn parse<'a>(
        &self,
        state: &mut ParseState,
        input: &'a [u8],
        grammar: &'a crate::Grammar,
    ) -> DecodeResult<'a, Arc<crate::DerivationTree>> {
        let dependent_node = state
            .context
            .get(&self.dep_name)
            .ok_or(DecodeError::Invalid(
                format!("Switch variable '{}' not found in context", self.dep_name).into(),
            ))?;

        // Just assume value is big endian
        let value = big_endian_bytes_to_usize(&dependent_node.to_bytes())?;

        let target_label_opt = state
            .get_enum(&self.dep_name)
            .and_then(|mapping| match mapping {
                EnumMapping::Int(map) => map.get(&value),
            });

        let target_label = target_label_opt
            .or(self.default_label.as_ref())
            .ok_or_else(|| {
                if state.get_enum(&self.dep_name).is_none() {
                    DecodeError::Invalid(
                        format!(
                            "Enum definition for '{}' not found and no default label provided",
                            self.dep_name
                        )
                        .into(),
                    )
                } else {
                    DecodeError::Invalid(
                        format!("No match for value {} in switch '{}'", value, self.dep_name)
                            .into(),
                    )
                }
            })?;

        nt(target_label).parse(state, input, grammar)
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
