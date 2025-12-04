//! Parser State

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use crate::DerivationTree;

/// Mapping Enums
#[derive(Debug, Clone)]
pub enum EnumMapping {
    /// Number as key
    Int(BTreeMap<usize, String>),
}

/// Global Parse State
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParseState {
    /// Context, which is mutable
    pub context: BTreeMap<String, Arc<DerivationTree>>,
    enums: Arc<HashMap<String, EnumMapping>>,
}

impl ParseState {
    /// Create a ParseState
    pub fn new(enums: Arc<HashMap<String, EnumMapping>>) -> Self {
        Self {
            context: BTreeMap::new(),
            enums,
        }
    }

    /// Get Enum
    pub fn get_enum(&self, case_name: &str) -> Option<&EnumMapping> {
        self.enums.get(case_name)
    }
}
