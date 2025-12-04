//! Useful traits in Grammar.

use crate::{Grammar, PangLabel, exp, nt_nom, t_bytes};

/// ToGrammar trait convert a type to a grammar
pub trait ToGrammar: PangLabel {
    /// Grammar of the type
    fn grammar() -> Grammar;
}

macro_rules! impl_to_grammar_for_primitive {
    ($($t:ty, $size:expr),*) => {
        $(
            impl ToGrammar for $t {
                fn grammar() -> Grammar {
                    use crate::{exp, t_bytes, Grammar};
                    let mut rules = Grammar::new();
                    rules.insert(Self::label().to_string(), vec![exp(vec![t_bytes($size)])]);
                    rules
                }
            }
        )*
    }
}

// Primitive types
impl_to_grammar_for_primitive!(
    u8, 1, u16, 2, u32, 4, u64, 8, u128, 16, i8, 1, i16, 2, i32, 4, i64, 8, i128, 16
);

// bool
impl ToGrammar for bool {
    fn grammar() -> Grammar {
        let mut rules = Grammar::new();
        rules.insert(Self::label(), vec![exp(vec![t_bytes(1)])]);
        rules
    }
}

// &T: ?Sized
impl<T: ToGrammar + ?Sized> ToGrammar for &T {
    fn grammar() -> Grammar {
        T::grammar()
    }
}

// Option<T>
impl<T: ToGrammar> ToGrammar for Option<T> {
    fn grammar() -> Grammar {
        let mut rules = Grammar::new();

        rules = rules.extend_grammar(&T::grammar());

        let vec_rule = vec![exp(vec![nt_nom(&T::label(), 0)])];

        rules.insert(Self::label().to_string(), vec_rule);
        rules
    }
}

// [T]
impl<T: ToGrammar> ToGrammar for [T] {
    fn grammar() -> Grammar {
        let mut rules = Grammar::new();

        rules = rules.extend_grammar(&T::grammar());

        let vec_rule = vec![exp(vec![nt_nom(&T::label(), 0)])];

        rules.insert(Self::label().to_string(), vec_rule);
        rules
    }
}

// Vec<T>
impl<T: ToGrammar> ToGrammar for Vec<T> {
    fn grammar() -> Grammar {
        <[T]>::grammar()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_to_grammar() {
        let grammar = u8::grammar();
        assert_eq!(grammar.get("u8").unwrap().len(), 1);
        let expansion = &grammar.get("u8").unwrap()[0];
        assert_eq!(expansion.symbols.len(), 1);
        assert_eq!(expansion.symbols[0], t_bytes(1));
    }

    #[test]
    fn test_u8_slice_to_grammar() {
        let grammar = <&[u8]>::grammar();
        let grammar_name = "[u8]";

        assert_eq!(grammar.get(grammar_name).unwrap().len(), 1);
        let expansion = &grammar.get(grammar_name).unwrap()[0];
        assert_eq!(expansion.symbols.len(), 1);
        assert_eq!(expansion.symbols[0], nt_nom("u8", 0));
    }

    #[test]
    fn test_option_u8_slice_to_grammar() {
        let grammar = <Option<&[u8]>>::grammar();
        let grammar_name = "Option<[u8]>";

        assert_eq!(grammar.get(grammar_name).unwrap().len(), 1);
        let expansion = &grammar.get(grammar_name).unwrap()[0];
        assert_eq!(expansion.symbols.len(), 1);
        assert_eq!(expansion.symbols[0], nt_nom("[u8]", 0));
    }

    #[test]
    fn test_vec_t_to_grammar() {
        // FIXME: Vec<u8> should be t_dyn like ToTree
        let grammar = <Vec<u8>>::grammar();
        let grammar_name = "[u8]";
        assert_eq!(grammar.get(grammar_name).unwrap().len(), 1);
        let expansion = &grammar.get(grammar_name).unwrap()[0];
        assert_eq!(expansion.symbols.len(), 1);
        assert_eq!(expansion.symbols[0], nt_nom("u8", 0));

        let grammar = <Vec<u64>>::grammar();
        let grammar_name = "[u64]";
        assert_eq!(grammar.get(grammar_name).unwrap().len(), 1);
        let expansion = &grammar.get(grammar_name).unwrap()[0];
        assert_eq!(expansion.symbols.len(), 1);
        assert_eq!(expansion.symbols[0], nt_nom("u64", 0));
    }
}
