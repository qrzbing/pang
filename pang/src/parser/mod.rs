//! Parser can parse input string with given grammar.

use std::hash::{Hash, Hasher};

mod combinator;
pub mod expansion;
pub mod factory;

/// Region contains a range of positions that is partially parsed.
#[derive(Debug, Clone, Copy, Eq)]
pub struct Region {
    /// Start position of the region.
    pub start: usize,
    /// End position of the region.
    pub end: usize,
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
