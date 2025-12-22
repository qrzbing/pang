use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

use crate::utils::{
    get_fields_processing_struct, get_pang_bitfield_attrs, get_semantic_type_from_attrs,
    get_struct_groups,
};

pub fn generate_field_to_tree(
    accessor: &proc_macro2::TokenStream,
    attrs: &[syn::Attribute],
) -> proc_macro2::TokenStream {
    let semantic_wrapper = get_semantic_type_from_attrs(attrs);
    if let Some(wrapper) = semantic_wrapper {
        quote! { <#wrapper>::from(#accessor.clone()).to_tree() }
    } else {
        quote! { #accessor.to_tree() }
    }
}

pub fn derive_to_tree_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    // Get the name of the struct/enum (e.g., "Header")
    let struct_name = &input.ident;
    let struct_label = struct_name.to_string();

    // Split generics (e.g., <T>) to ensure the generated `impl` block compiles correctly.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generate the body of the `to_tree()` function
    let body =
        match &input.data {
            // ====
            // CASE: Struct
            // Example: struct Header { ver: u8, len: u16 }
            // Target Tree: Node("Header")
            //              |-> Node("Header.ver") -> [ValueNode]
            //              |-> Node("Header.len") -> [ValueNode]
            // ====
            Data::Struct(data) => {
                // Extract all fields (named or unnamed) standardizing access (self.field).
                let raw_fields = get_fields_processing_struct(data);

                let groups = get_struct_groups(data);

                let nodes = groups.iter().map(|group| {
                let node_name = format!("{}.{}", struct_label, group.label);

                if group.is_bitfield {
                    // === Generate bitfield ===
                    let container_ty = if group.max_bits <= 8 { quote!{ u8 } }
                                       else if group.max_bits <= 16 { quote!{ u16 } }
                                       else if group.max_bits <= 32 { quote!{ u32 } }
                                       else { quote!{ u64 } };

                    let merge_steps = group.members.iter().map(|&idx| {
                        let (accessor, _, attrs, _) = &raw_fields[idx];
                        let bf = get_pang_bitfield_attrs(attrs).unwrap();
                        let mask = (1u128 << (bf.end - bf.start)) - 1;
                        let mask_lit = proc_macro2::Literal::u128_unsuffixed(mask);
                        let shift_lit = proc_macro2::Literal::usize_unsuffixed(bf.start);

                        quote! {
                            accumulator |= ((#accessor as #container_ty) & #mask_lit) << #shift_lit;
                        }
                    });

                    quote! {
                        {
                            let mut accumulator: #container_ty = 0;
                            #(#merge_steps)*
                            pang::new_node(pang::nt(#node_name), Some(vec![accumulator.to_tree()]))
                        }
                    }
                } else {
                    // === Generate normal fields ===
                    let idx = group.members[0];
                    let (accessor, _, attrs, _) = &raw_fields[idx];
                    let value_tree = crate::tree::generate_field_to_tree(accessor, attrs);
                    quote! {
                        pang::new_node(pang::nt(#node_name), Some(vec![#value_tree]))
                    }
                }
            });

                quote! {
                    let children = vec![ #(#nodes),* ];
                    pang::new_node(pang::nt(#struct_label), Some(children))
                }
            }
            Data::Enum(data) => {
                let match_arms = data.variants.iter().map(|variant| {
                    let variant_ident = &variant.ident;

                    let (pattern, accessors, field_attrs_list) = match &variant.fields {
                        Fields::Named(fields) => {
                            let names: Vec<_> = fields
                                .named
                                .iter()
                                .map(|f| f.ident.clone().unwrap())
                                .collect();
                            let attrs: Vec<_> = fields.named.iter().map(|f| &f.attrs).collect();
                            (
                                quote! { Self::#variant_ident { #(#names),* } },
                                names.iter().map(|n| quote! { #n }).collect::<Vec<_>>(),
                                attrs,
                            )
                        }
                        Fields::Unnamed(fields) => {
                            let vars: Vec<_> = (0..fields.unnamed.len())
                                .map(|i| format_ident!("_{}", i))
                                .collect();
                            let attrs: Vec<_> = fields.unnamed.iter().map(|f| &f.attrs).collect();
                            (
                                quote! { Self::#variant_ident ( #(#vars),* ) },
                                vars.iter().map(|v| quote! { #v }).collect::<Vec<_>>(),
                                attrs,
                            )
                        }
                        Fields::Unit => (quote! { Self::#variant_ident }, vec![], vec![]),
                    };

                    let child_conversions = accessors
                        .iter()
                        .zip(field_attrs_list.iter())
                        .map(|(acc, attrs)| generate_field_to_tree(acc, attrs));

                    quote! {
                        #pattern => {
                            let children = vec![
                                #(#child_conversions),*
                            ];
                            pang::new_node(pang::nt(#struct_label), Some(children))
                        }
                    }
                });

                quote! {
                    match self {
                        #(#match_arms),*
                    }
                }
            }
            _ => panic!("ToTree only supports Struct and Enum"),
        };

    quote! {
        impl #impl_generics pang::ToTree for #struct_name #ty_generics #where_clause {
            fn to_tree(&self) -> std::sync::Arc<pang::DerivationTree> {
                #body
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_to_tree_middleware_naming() {
        let input: DeriveInput = parse_quote! {
            struct Header { ver: u8, len: u16 }
        };
        let output = derive_to_tree_impl(input);
        let s = output.to_string();
        // This will generate middleware `Header.ver` and `Header.len`
        assert!(s.contains("pang :: nt (\"Header.ver\")"));
        assert!(s.contains("pang :: nt (\"Header.len\")"));
    }
}
