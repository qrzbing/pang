/// Get filed node from struct
#[macro_export]
macro_rules! field_node {
    ($prefix:expr, $self:ident, $name:ident) => {
        new_node(
            nt(&format!("{}.{}", $prefix, stringify!($name))),
            Some(vec![$self.$name.to_tree()]),
        )
    };

    ($prefix:expr, $name_str:literal, $val:expr) => {
        new_node(
            nt(&format!("{}.{}", $prefix, $name_str)),
            Some(vec![$val.to_tree()]),
        )
    };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{DerivationTree, PangLabel, ToTree, new_node, nt};

    #[test]
    fn test_macro_field_node() {
        struct Test {
            a: bool,
            b: u8,
            c: u32,
        }

        impl PangLabel for Test {
            fn label() -> String {
                "Test".to_string()
            }
        }

        impl ToTree for Test {
            fn to_tree(&self) -> Arc<DerivationTree> {
                let b_fixed = self.b & 0x7F;
                let children = vec![
                    field_node!("Test", self, a),
                    field_node!("Test", "b_fixed", b_fixed),
                    field_node!("Test", self, c),
                ];
                new_node(nt("Test"), Some(children))
            }
        }

        let test = Test {
            a: true,
            b: 0x12,
            c: 0xdeadbeef,
        };
        let node = field_node!("Test", test, a);

        assert_eq!(node.symbol, nt("Test.a"));

        assert_eq!(
            test.to_tree().to_bytes(),
            [1, 0x12 & 0x7F, 0xde, 0xad, 0xbe, 0xef]
        )
    }
}
