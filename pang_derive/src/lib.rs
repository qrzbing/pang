use syn::{DeriveInput, parse_macro_input};

extern crate proc_macro;
use proc_macro::TokenStream;

use crate::{
    grammar::derive_to_grammar_impl, label::derive_pang_label_impl, tree::derive_to_tree_impl,
};

mod grammar;
mod label;
mod tree;
mod utils;

#[proc_macro_derive(PangLabel, attributes(pang))]
pub fn derive_pang_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_pang_label_impl(input).into()
}

#[proc_macro_derive(ToTree, attributes(pang))]
pub fn derive_to_tree(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_to_tree_impl(input).into()
}

#[proc_macro_derive(ToGrammar, attributes(pang))]
pub fn derive_to_grammar(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_to_grammar_impl(input).into()
}
