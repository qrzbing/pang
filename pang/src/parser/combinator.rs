//!

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    grammar::{Expansion, Grammar},
    parser::factory::get_expansion_parser,
    symbol::{DecodeError, DecodeResult, SharedState, Symbol, nt},
    tree::{DerivationTree, new_node},
};

impl Grammar {
    ///
    pub fn parse_combinator<'a>(
        &'a self,
        input: &'a [u8],
        start_symbol: &str,
    ) -> Result<Arc<DerivationTree>, DecodeError> {
        let start_node = nt(start_symbol);
        let mut context = BTreeMap::new();

        match start_node.parse(input, self, &mut context) {
            Ok((remaining, tree)) => {
                if !remaining.is_empty() {
                    println!(
                        "Warning: Input not fully consumed. {} bytes remaining.",
                        remaining.len()
                    );
                }
                Ok(tree)
            }
            Err(e) => Err(e),
        }
    }
}

impl Symbol {
    ///
    pub fn parse<'a>(
        &'a self,
        input: &'a [u8],
        grammar: &'a Grammar,
        context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Arc<DerivationTree>> {
        match self {
            // Parse NonTerminal
            Symbol::NonTerminal { label } => {
                let expansions = grammar
                    .get(label)
                    .ok_or(DecodeError::Invalid("Non-terminal not found in grammar"))?;

                for expansion in expansions {
                    let parser = get_expansion_parser(expansion);

                    if let Ok((remaining_input, children)) =
                        parser.parse(input, expansion, grammar, context)
                    {
                        let node = new_node(self.clone(), Some(children));
                        context.insert(label.clone(), node.clone());
                        return Ok((remaining_input, node));
                    }
                }
                Err(DecodeError::Invalid("No expansion matched for NonTerminal"))
            }
            // Parse Terminal
            Symbol::Terminal { kind } => {
                let (remaining_input, new_kind) =
                    kind.parse(input, &SharedState::new(), context)?;
                let new_symbol = Symbol::Terminal { kind: new_kind };
                let node = new_node(new_symbol, Some(vec![]));
                Ok((remaining_input, node))
            }
        }
    }
}

impl Expansion {
    /// Parse an Expansion
    pub fn parse<'a>(
        &'a self,
        input: &'a [u8],
        grammar: &'a Grammar,
        parent_context: &mut BTreeMap<String, Arc<DerivationTree>>,
    ) -> DecodeResult<'a, Vec<Arc<DerivationTree>>> {
        let mut remaining_input = input;
        let mut children = Vec::new();
        // Create a local context
        let mut local_context = parent_context.clone();

        // Parse each symbol in the expansion
        for symbol in &self.symbols {
            // Call Symbol::parse with the remaining input and the local context
            match symbol.parse(remaining_input, grammar, &mut local_context) {
                Ok((next_input, child_node)) => {
                    remaining_input = next_input;
                    children.push(child_node);
                }
                Err(e) => {
                    // If any symbol fails, return the error
                    return Err(e);
                }
            }
        }

        // If all symbols parsed successfully, return the children nodes
        Ok((remaining_input, children))
    }
}
