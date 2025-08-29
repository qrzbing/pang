//! [`Generator`] can generate random derivation trees from a given [`Grammar`].

use std::sync::Arc;

use crate::{
    grammar::{Expansion, Grammar},
    symbol::{Symbol, nt, terminals::DynRand},
    tree::{DerivationTree, TreeFixer, new_node},
};

/// Generator for [`Grammar`].
impl Grammar {
    ///
    pub fn generate_combinator(
        &self,
        start_symbol: &str,
        rng: &mut dyn DynRand,
        fixers: &[Arc<dyn TreeFixer>],
    ) -> Arc<DerivationTree> {
        let start_node = nt(start_symbol);

        // 1. Generate the basic tree structure.
        let generated_tree = start_node
            .generate(self, rng)
            .expect(&format!("Generation from {} should not fail", start_symbol));

        // 2. Apply fixers to fix context-dependent values (e.g., length, etc.).
        let mut fixed_tree = generated_tree;
        for fixer in fixers {
            fixed_tree = fixer.fix(self, fixed_tree);
        }

        fixed_tree
    }
}

/// Generator for [`Symbol`].
impl Symbol {
    /// Generate a tree for this symbol.
    pub fn generate(
        &self,
        grammar: &Grammar,
        rng: &mut dyn DynRand,
    ) -> Result<Arc<DerivationTree>, String> {
        match self {
            // 1. Expanse for NonTerminal
            Symbol::NonTerminal { label } => {
                let expansions = grammar
                    .get(label)
                    .ok_or_else(|| format!("Non-terminal '{}' not found in grammar", label))?;

                if expansions.is_empty() {
                    return Err(format!("No expansions available for '{}'", label));
                }

                let chosen_expansion = &expansions[rng.below_or_zero(expansions.len())];

                let children = chosen_expansion.generate(grammar, rng)?;
                let node = new_node(self.clone(), Some(children));
                Ok(node)
            }

            Symbol::Terminal { kind } => Ok(kind.generate(rng)),
        }
    }
}

/// Generator for [`Expansion`].
impl Expansion {
    /// Generate a list of nodes for this expansion.
    pub fn generate(
        &self,
        grammar: &Grammar,
        rng: &mut dyn DynRand,
    ) -> Result<Vec<Arc<DerivationTree>>, String> {
        let mut children = Vec::new();

        // Generate each symbol in the expansion.
        for symbol in &self.symbols {
            let child_node = symbol.generate(grammar, rng)?;
            children.push(child_node);
        }

        Ok(children)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Once;

    use libafl_bolts::rands::StdRand;

    use crate::grammar::{
        asn1_tlv_grammar,
        examples::tlv::{nest_tlv_grammar, tlv_grammar},
    };

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            let _ = env_logger::try_init();
        });
    }

    #[test]
    fn test_generate_tlv_grammar() {
        setup_logger();
        let mut rng = StdRand::with_seed(0);
        let tree = tlv_grammar().generate_combinator("start", &mut rng, &[]);
        println!("Generated tree: {}", tree);
        println!("Generated tree: {:?}", tree.to_bytes());
    }

    #[test]
    fn test_generate_nest_tlv_grammar() {
        setup_logger();
        let mut rng = StdRand::with_seed(0);
        let tree = nest_tlv_grammar().generate_combinator("start", &mut rng, &[]);
        println!("Generated tree: {}", tree);
    }

    #[test]
    fn test_generate_asn1_grammar() {
        setup_logger();
        let mut rng = StdRand::with_seed(0);
        let tree = asn1_tlv_grammar().generate_combinator("asn1-tlv", &mut rng, &[]);
        println!("Generated tree: {}", tree);
    }
}
