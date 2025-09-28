//! Generator can generate random derivation trees from a given [`Grammar`].

use std::sync::Arc;

use crate::{
    grammar::{Expansion, Grammar},
    symbol::{Symbol, nt, terminals::DynRand},
    tree::{DerivationTree, new_node},
};

/// Generator for [`Grammar`].
impl Grammar {
    ///
    pub fn generate_combinator(
        &self,
        start_symbol: &str,
        rng: &mut dyn DynRand,
    ) -> Arc<DerivationTree> {
        nt(start_symbol)
            .generate(self, rng)
            .expect(&format!("Generation from {} should not fail", start_symbol))
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
            Symbol::NonTerminal { kind } => kind.generate(grammar, rng),
            Symbol::Terminal { label, kind } => Ok(new_node(
                Symbol::Terminal {
                    label: label.clone(),
                    kind: kind.generate(rng),
                },
                Some(vec![]),
            )),
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

    use crate::language::{asn1_tlv_lang, examples::tlv::tlv_lang};

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            let _ = env_logger::try_init();
        });
    }

    #[test]
    fn test_generate_tlv_lang() {
        setup_logger();
        let mut rng = StdRand::with_seed(1);
        let grammar = tlv_lang().grammar;
        let tree = grammar.generate_combinator("start", &mut rng);
        let len_terminal_val = tree
            .at(&[0, 1])
            .unwrap()
            .first_terminal_kind()
            .unwrap()
            .as_has_length()
            .unwrap()
            .as_length()
            .unwrap();
        let val_terminal_len = tree.at(&[0, 2]).unwrap().to_bytes().len();
        assert_eq!(len_terminal_val, val_terminal_len);
    }

    #[test]
    fn test_generate_asn1_lang() {
        setup_logger();
        let mut rng = StdRand::with_seed(0);
        let tree = asn1_tlv_lang()
            .grammar
            .generate_combinator("asn1-tlv", &mut rng);
        let len = tree
            .at(&[1])
            .unwrap()
            .first_terminal_kind()
            .unwrap()
            .as_has_length()
            .unwrap()
            .as_length()
            .unwrap();
        let val_len = tree.at(&[2]).unwrap().to_bytes().len();
        assert_eq!(len, val_len);
    }
}
