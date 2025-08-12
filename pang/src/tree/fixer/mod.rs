//! TreeFixer can fix DerivationTree by anything you want.

use crate::{grammar::Grammar, tree::DerivationTree};
use std::sync::Arc;

/// TreeFixer trait
pub trait TreeFixer: std::fmt::Debug {
    /// Fix tree and subtree by fixer
    fn fix(&self, grammar: &Grammar, node: Arc<DerivationTree>) -> Arc<DerivationTree>;
}
