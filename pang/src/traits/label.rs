///
pub trait PangLabel {
    ///
    fn label() -> String;
}

macro_rules! impl_label_for_primitive {
    ($($t:ty),*) => {
        $(
            impl PangLabel for $t {
                fn label() -> String {
                    stringify!($t).to_string()
                }
            }
        )*
    }
}

impl_label_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, bool);

// &T: ?Sized
impl<T: PangLabel + ?Sized> PangLabel for &T {
    fn label() -> String {
        T::label()
    }
}

// Option<T>
impl<T: PangLabel> PangLabel for Option<T> {
    fn label() -> String {
        format!("Option<{}>", T::label())
    }
}

// [T]
impl<T: PangLabel> PangLabel for [T] {
    fn label() -> String {
        format!("[{}]", T::label())
    }
}

// Vec<T>
impl<T: PangLabel> PangLabel for Vec<T> {
    fn label() -> String {
        <[T]>::label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_label() {
        assert_eq!(u8::label(), "u8");
        assert_eq!(u128::label(), "u128");
        assert_eq!(i16::label(), "i16");
        assert_eq!(i64::label(), "i64");
        assert_eq!(bool::label(), "bool");
    }
}
