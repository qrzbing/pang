/*!
Welcome to `PANG`
*/

#![cfg_attr(
    not(test),
    warn(
        missing_debug_implementations,
        missing_docs,
        trivial_numeric_casts,
        unused_extern_crates,
        unused_import_braces,
        unused_qualifications,
    )
)]
#![cfg_attr(
    test,
    deny(
        bad_style,
        dead_code,
        improper_ctypes,
        missing_debug_implementations,
        missing_docs,
        no_mangle_generic_items,
        non_shorthand_field_patterns,
        overflowing_literals,
        path_statements,
        patterns_in_fns_without_body,
        trivial_numeric_casts,
        unconditional_recursion,
        unfulfilled_lint_expectations,
        unused_allocation,
        unused_comparisons,
        unused_extern_crates,
        unused_import_braces,
        unused_must_use,
        unused_parens,
        unused_qualifications,
        unused,
        while_true
    )
)]

mod generator;
pub mod grammar;
pub use grammar::*;
pub mod language;
pub use language::Language;
pub mod parser;
pub mod symbol;
pub use symbol::*;
pub mod tree;
pub use tree::{DerivationTree, new_node};
