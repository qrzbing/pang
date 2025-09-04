//! State stores some useful information during fuzzing.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    sync::Arc,
};

use ahash::AHasher;
use libafl_bolts::{impl_serdeany, rands::Rand};
use serde::{Deserialize, Serialize};

use pang::{grammar::Grammar, tree::DerivationTree};

use crate::input::PangInput;

const MAX_SEEN_SEEDS: usize = 10000;
const MAX_FRAGMENTS_PER_KEY: usize = 100;

/// [`PangMutateState`] stores useful information during mutation.
#[derive(Debug, Serialize, Deserialize)]
pub struct PangMutateState {
    /// Save fragments for each key.
    pub fragments: HashMap<String, Vec<Arc<DerivationTree>>>,
    /// Save seen seeds.
    pub seen_seeds: VecDeque<PangInput>,
    /// Save seen hashes for fast check.
    pub seen_hashes: HashSet<u64>,
}

impl PangMutateState {
    /// Create a new [`PangMutateState`] with the given parser.
    pub fn new(grammar: &Grammar) -> Self
where {
        let mut fragments = HashMap::new();
        for key in grammar.keys() {
            fragments.insert(key.to_string(), Vec::new());
        }
        Self {
            fragments,
            seen_seeds: VecDeque::with_capacity(MAX_SEEN_SEEDS),
            seen_hashes: HashSet::with_capacity(MAX_SEEN_SEEDS),
        }
    }

    /// Calculate PangInput Hash
    fn calculate_hash(input: &PangInput) -> u64 {
        let mut hasher = AHasher::default();
        input.hash(&mut hasher);
        hasher.finish()
    }

    /// Add a seed to the seen_seeds list.
    pub fn add_seen_seed(&mut self, input: &PangInput) -> bool {
        // If seen_hashed contains current seed, return.
        let seed_hash = Self::calculate_hash(&input);
        if self.seen_hashes.contains(&seed_hash) {
            return false;
        }

        // If seen_seeds is full, remove the oldest seed.
        if self.seen_seeds.len() >= MAX_SEEN_SEEDS {
            if let Some(old_seed) = self.seen_seeds.pop_front() {
                let old_hash = Self::calculate_hash(&old_seed);
                self.seen_hashes.remove(&old_hash);
            }
        }

        self.seen_seeds.push_back(input.clone());
        self.seen_hashes.insert(seed_hash);
        true
    }

    /// Add a fragment to the corresponding key in the fragments map.
    pub fn add_fragment<R: Rand>(
        &mut self,
        key: String,
        fragment: Arc<DerivationTree>,
        rng: &mut R,
    ) {
        let frags = self.fragments.entry(key).or_default();
        if frags.len() < MAX_FRAGMENTS_PER_KEY {
            frags.push(fragment);
        } else {
            let idx_to_replace = rng.below_or_zero(frags.len());
            frags[idx_to_replace] = fragment;
        }
    }
}
impl_serdeany!(PangMutateState);
