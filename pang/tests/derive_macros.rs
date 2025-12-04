use log::debug;
use pang::{PangLabel, ToGrammar, ToTree};

#[derive(ToTree, ToGrammar, PangLabel)]
struct A<'a> {
    b: B,
    c: u16,
    d: &'a [u8],
    e: Option<&'a [u8]>,
    f: Vec<C<'a>>,
}

#[derive(ToTree, ToGrammar, PangLabel)]
struct B(pub u8);

#[derive(ToTree, ToGrammar, PangLabel)]
struct C<'a> {
    a: &'a [u8],
}

#[derive(ToTree, ToGrammar, PangLabel)]
struct D<'a> {
    a: Vec<C<'a>>,
}

#[derive(ToTree, ToGrammar, PangLabel)]
#[allow(dead_code)]
enum E<'a> {
    A(A<'a>),
    B(B),
    C(C<'a>),
    D(D<'a>),
    E(u8),
    F,
}

#[test]
fn test_complex_struct_a() {
    let a = A {
        b: B(1),
        c: 2,
        d: &[],
        e: Some(&[1]),
        f: vec![C { a: &[1] }],
    };
    let tree = a.to_tree();
    debug!("Tree output:\n{}", tree);
    debug!("Tree output:\n{}", tree.to_string());
    assert!(tree.to_string().contains("<[C]>"));

    debug!("Grammar output:\n{}", A::grammar());
}

#[test]
fn test_tuple_struct_b() {
    let b = B(1);
    let tree = b.to_tree();
    debug!("{}", tree);
    debug!("{}", B::grammar())
}

#[test]
fn test_nested_vec_d() {
    let d = D {
        a: vec![C { a: &[] }],
    };
    let tree = d.to_tree();
    debug!("{}", tree);
    debug!("{}", D::grammar())
}

#[test]
fn test_enum_e() {
    let e = E::B(B(1));
    let tree = e.to_tree();
    debug!("Tree (Enum::B):\n{}", tree);

    let e_f = E::F;
    let tree_f = e_f.to_tree();
    debug!("Tree (Enum::F):\n{}", tree_f);
}
