use std::{ops::Deref, sync::Arc};

use crate::{
    DerivationTree, Grammar, PangLabel, ToGrammar, ToTree, exp, new_node, t_bytes, t_bytes_val,
};

macro_rules! define_endian_type {
    ($name:ident, $inner:ty, $size:expr, $method:ident) => {
        /// $name as a primitive type
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
        pub struct $name(pub $inner);

        impl PangLabel for $name {
            fn label() -> String {
                stringify!($name).to_string()
            }
        }

        impl ToGrammar for $name {
            fn grammar() -> Grammar {
                let mut rules = Grammar::new();
                rules.insert(Self::label(), vec![exp(vec![t_bytes($size)])]);
                rules
            }
        }

        impl ToTree for $name {
            fn to_tree(&self) -> Arc<DerivationTree> {
                new_node(t_bytes_val(&self.0.$method()), None)
            }
        }

        impl Deref for $name {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(v: $inner) -> Self {
                $name(v)
            }
        }

        impl From<$name> for $inner {
            fn from(v: $name) -> Self {
                v.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_endian_type!(U16be, u16, 2, to_be_bytes);
define_endian_type!(U32be, u32, 4, to_be_bytes);
define_endian_type!(U64be, u64, 8, to_be_bytes);

define_endian_type!(U16le, u16, 2, to_le_bytes);
define_endian_type!(U32le, u32, 4, to_le_bytes);
define_endian_type!(U64le, u64, 8, to_le_bytes);
