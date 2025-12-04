use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

use crate::utils::{get_field_label, get_fields_processing_struct, get_semantic_type_from_attrs};

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
    let body = match &input.data {
        // ====
        // CASE: Struct
        // Example: struct Header { ver: u8, len: u16 }
        // Target Tree: Node("Header")
        //              |-> Node("Header.ver") -> [ValueNode]
        //              |-> Node("Header.len") -> [ValueNode]
        // ====
        Data::Struct(data) => {
            // Extract all fields (named or unnamed) standardizing access (self.field).
            let fields_processing = get_fields_processing_struct(data);

            // Iterate over every field to generate code that converts it into a sub-tree node.
            let to_tree_body = fields_processing.iter().enumerate().map(
                |(idx, (accessor, _ty, attrs, field_name_opt))| {
                    // 1. Naming the Intermediate Node
                    // Result: "Header.ver" or "Header.len"
                    let field_label = get_field_label(field_name_opt, idx);
                    let unique_node_name = format!("{}.{}", struct_label, field_label);

                    // 2. Recursion for children
                    let value_tree = generate_field_to_tree(accessor, attrs);

                    // 3. Code Generation for this specific field
                    quote! {
                        pang::new_node(
                            pang::nt(#unique_node_name),
                            Some(vec![ #value_tree ])
                        )
                    }
                },
            );

            // 4. Assemble the Struct
            // Combine all field nodes into a list and wrap them in the parent Struct Node.
            quote! {
                let children = vec![
                    #(#to_tree_body),*
                ];
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
