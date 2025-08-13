//! Input type for Pang, compatible with LibAFL

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs::File,
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::Path,
    sync::Arc,
};

use libafl::inputs::{HasTargetBytes, Input};
use libafl_bolts::{Error, ownedref::OwnedSlice};
use serde::{Deserialize, Serialize};

use pang::{parser::Region, tree::DerivationTree};

/// [`PangInput`] has a [`DerivationTree`] structure that can be use for grammar fuzzing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PangInput {
    /// [`PangInput`] can be created from bytes or convert to bytes.
    pub bytes: Vec<u8>,
    /// `structure` is a [`DerivationTree`] that is generated from parsing.
    #[serde(skip)] // The tree is runtime state, not part of serialization
    pub structure: Option<Arc<DerivationTree>>,
    /// If [`PangInput`] can be parsed, `has_structure` is true.
    #[serde(skip)]
    pub has_structure: bool,
    /// If [`PangInput`] can not be fully parsed, `regions` contains the partially parsed regions.
    #[serde(skip)]
    pub regions: Option<HashMap<String, HashSet<Region>>>,
    /// If [`PangInput`] has regions, `has_regions` is true.
    #[serde(skip)]
    pub has_regions: bool,
}

impl Hash for PangInput {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl PartialEq for PangInput {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for PangInput {}

impl Input for PangInput {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(Self {
            bytes,
            structure: None,
            has_structure: false,
            regions: None,
            has_regions: false,
        })
    }

    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut file = File::create(path)?;
        Ok(file.write_all(&self.bytes)?)
    }
}

impl HasTargetBytes for PangInput {
    fn target_bytes(&self) -> OwnedSlice<u8> {
        (&self.bytes).into()
    }
}
