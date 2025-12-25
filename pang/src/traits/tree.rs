//! Useful traits in Tree.

use std::sync::Arc;

use crate::{DerivationTree, PangLabel, new_node, nt, nt_nom, t_bytes_val};

/// ToTree trait convert a type to a derivation tree
pub trait ToTree: PangLabel {
    /// Convert a type to a derivation tree
    fn to_tree(&self) -> Arc<DerivationTree>;
}

/// Implement ToTree for primitive types
macro_rules! impl_to_tree_for_primitive {
    ($($t:ty),*) => {
        $(
            impl ToTree for $t {
                fn to_tree(&self) -> Arc<DerivationTree> {
                    use crate::{new_node, t_bytes_val};
                    new_node(t_bytes_val(&self.to_be_bytes()), None)
                }
            }
        )*
    }
}

impl_to_tree_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl ToTree for bool {
    fn to_tree(&self) -> Arc<DerivationTree> {
        // change bool to u8, true -> 1, false -> 0
        let val: u8 = if *self { 1 } else { 0 };
        new_node(t_bytes_val(&[val]), None)
    }
}

impl<T: ToTree> ToTree for Option<T> {
    fn to_tree(&self) -> Arc<DerivationTree> {
        let children = match self {
            Some(v) => vec![v.to_tree()],
            None => vec![],
        };

        new_node(nt(&Self::label()), Some(children))
    }
}

impl<T: ToTree> ToTree for [T] {
    fn to_tree(&self) -> Arc<DerivationTree> {
        let children: Vec<_> = self.iter().map(|i| i.to_tree()).collect();
        let repetition_node = new_node(nt_nom(&T::label(), 0), Some(children));
        new_node(nt(&Self::label()), Some(vec![repetition_node]))
    }
}

impl<T: ToTree> ToTree for Vec<T> {
    fn to_tree(&self) -> Arc<DerivationTree> {
        self.as_slice().to_tree()
    }
}

// Size allow T as Dynamically Sized Type
impl<T: ToTree + ?Sized> ToTree for &T {
    fn to_tree(&self) -> Arc<DerivationTree> {
        (**self).to_tree()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_to_tree() {
        let u8_val: u8 = 0xff;
        let u8_tree = u8_val.to_tree();
        assert_eq!(u8_tree.to_bytes(), [0xff]);

        let u32_val: u32 = 0xdeadbeef;
        let u32_tree = u32_val.to_tree();
        assert_eq!(u32_tree.to_bytes(), u32_val.to_be_bytes());
    }

    #[test]
    fn test_u8_slice_to_tree() {
        let u8_slice = [u8::MAX, u8::MIN];
        let u8_slice_tree = u8_slice.to_tree();
        assert_eq!(u8_slice_tree.to_bytes(), u8_slice);

        let u8_vec = vec![u8::MAX, u8::MIN];
        let u8_vec_tree = u8_vec.to_tree();
        assert_eq!(u8_vec_tree.to_bytes(), u8_vec);
    }

    #[test]
    fn test_option_to_tree() {
        let option_val: Option<u8> = Some(0xff);
        let option_tree = option_val.to_tree();
        assert_eq!(option_tree.to_bytes(), [0xff]);

        let option_val: Option<u8> = None;
        let option_tree = option_val.to_tree();
        assert_eq!(option_tree.to_bytes(), []);
    }
}
