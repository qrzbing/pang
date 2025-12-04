use quote::quote;
use syn::{Data, DeriveInput, Fields};

use crate::utils::{
    get_field_label, get_fields_processing_struct, get_semantic_type_from_attrs,
    get_type_override_from_attrs,
};

pub fn derive_to_grammar_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &input.ident; // The name of the type (e.g., "MyStruct")
    let struct_label = struct_name.to_string();

    // Split generics for use in `impl <...> ...`
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generate the body of the `grammar()` function based on data type
    let grammar_body = match &input.data {
        Data::Struct(data) => {
            // Gets fields, names, attributes
            let fields_processing = get_fields_processing_struct(data);

            let mut main_seq = vec![]; // Tokens for the main rule: [nt("S.f1"), nt("S.f2")]
            let mut extra_rules = vec![]; // Definition logic for fields

            for (idx, (_accessor, ty, attrs, field_name_opt)) in
                fields_processing.iter().enumerate()
            {
                // 1. Create a unique Label for this field's rule
                // e.g., "MyStruct.username" or "MyTuple.0"
                let field_label = get_field_label(field_name_opt, idx);
                let unique_rule_name = format!("{}.{}", struct_label, field_label);

                // 2. Add reference to the Main Sequence
                // The main struct rule will point to this intermediate rule.
                main_seq.push(quote! { pang::nt(#unique_rule_name) });

                // 3. Determine the logic for the field (Recursion vs Override)
                let (pre_logic, rule_content) = if let Some(custom_expr) =
                    get_type_override_from_attrs(attrs)
                {
                    // Case A: User Override (e.g., #[pang(type="Digit")])
                    // No recursion needed, just output the custom expression.
                    (quote! {}, quote! { vec![pang::#custom_expr] })
                } else {
                    // Case B: Default Behavior (Type Recursion)

                    // Check if user remapped the semantic type (e.g., #[pang(semantic="MyStruct")])
                    let semantic_wrapper = get_semantic_type_from_attrs(attrs);
                    let target_type = if let Some(wrapper) = semantic_wrapper {
                        quote! { #wrapper } // Use "MyStruct"
                    } else {
                        quote! { #ty } // Use default type
                    };

                    (
                        // pre_logic: RECURSION step.
                        // Call `grammar()` on the child type and merge its rules into ours.
                        // This ensures that if `MyStruct` contains `InnerStruct`,
                        // `InnerStruct`'s rules are also included.
                        quote! {
                            rules = rules.extend_grammar(&<#target_type as pang::ToGrammar>::grammar());
                        },
                        // rule_content: Point to the child type's label.
                        quote! {
                            vec![pang::nt(&<#target_type as pang::PangLabel>::label())]
                        },
                    )
                };

                // 4. Register the Intermediate Rule
                extra_rules.push(quote! {
                    #pre_logic // Execute side effects (merging sub-grammars)

                    // Insert the rule: "MyStruct.field1" -> [ "String" ]
                    rules.insert(#unique_rule_name.to_string(), vec![pang::exp(#rule_content)]);
                });
            }

            quote! {
                use pang::{exp, nt};
                // Insert Main Rule: "MyStruct" -> [ "MyStruct.field1", "MyStruct.field2" ]
                rules.insert(#struct_label.to_string(), vec![exp(vec![
                    #(#main_seq),*
                ])]);

                // Execute field definitions
                #(#extra_rules)*
            }
        }
        Data::Enum(data) => {
            let variants_process_logic = data.variants.iter().map(|variant| {
                let variant_ident = &variant.ident;
                // Label for the variant, e.g., "MyEnum.VariantA"
                let variant_label = format!("{}.{}", struct_label, variant_ident);

                // Extract fields
                let fields = match &variant.fields {
                    Fields::Named(f) => f.named.iter().collect::<Vec<_>>(),
                    Fields::Unnamed(f) => f.unnamed.iter().collect::<Vec<_>>(),
                    Fields::Unit => vec![], // TODO: support Unit
                };

                // Iterate fields inside the variant (e.g., MyEnum::VariantA(u8, u8))
                let (variant_extends, variant_symbols): (Vec<_>, Vec<_>) = fields
                    .iter()
                    .map(|f| {
                        let ty = &f.ty;
                        let attrs = &f.attrs;

                        // Same recursion/override logic as Structs
                        if let Some(custom_expr) = get_type_override_from_attrs(attrs) {
                            (quote! {}, quote! { pang::#custom_expr })
                        } else {
                            let semantic_wrapper = get_semantic_type_from_attrs(attrs);
                            let target_type = if let Some(wrapper) = semantic_wrapper {
                                quote! { #wrapper }
                            } else {
                                quote! { #ty }
                            };

                            // Returns:
                            // 1. Side effect code (merging sub-grammars)
                            // 2. The symbol to add to this variant's sequence
                            (
                                quote! {
                                    rules = rules.extend_grammar(&<#target_type as pang::ToGrammar>::grammar());
                                },
                                quote! {
                                    pang::nt(&<#target_type as pang::PangLabel>::label())
                                },
                            )
                        }
                    })
                    .unzip();

                // Code block for THIS specific variant
                quote! {
                    {
                        // 1. Merge sub-grammars for fields in this variant
                        #(#variant_extends)*

                        // 2. Build the sequence for this variant
                        // e.g., VariantA -> [ u8, u8 ]
                        let variant_seq = vec![ #(#variant_symbols),* ];

                        // 3. Register the Variant's rule definition
                        rules.insert(#variant_label.to_string(), vec![pang::exp(variant_seq)]);

                        // 4. Add this variant as a choice to the parent Enum's list
                        // Pushes "MyEnum.VariantA" into the `alternatives` vector.
                        alternatives.push(pang::exp(vec![ pang::nt(#variant_label) ]));
                    }
                }
            });

            quote! {
                use pang::{exp, nt};
                let mut alternatives = vec![];

                // Process all variants
                #(#variants_process_logic)*

                // Insert Main Enum Rule: "MyEnum" -> ( VariantA | VariantB | ... )
                rules.insert(#struct_label.to_string(), alternatives);
            }
        }
        _ => panic!("ToGrammar only supports Struct and Enum"),
    };

    // Wrap everything in the Trait Implementation
    quote! {
        impl #impl_generics pang::ToGrammar for #struct_name #ty_generics #where_clause {
            fn grammar() -> pang::Grammar {
                use pang::Grammar;
                let mut rules = Grammar::new();
                #grammar_body   // Insert generated rules above
                rules
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_to_grammar_middleware_naming() {
        let input: DeriveInput = parse_quote! {
            struct Header {
                ver: u8,
                len: u16
            }
        };
        let output = derive_to_grammar_impl(input);
        let s = output.to_string();

        // As test above
        assert!(s.contains("pang :: nt (\"Header.ver\")"));
        assert!(s.contains("pang :: nt (\"Header.len\")"));

        // rules.extend_grammar(u8::grammar());
        assert!(s.contains("rules . extend_grammar (& < u8 as pang :: ToGrammar > :: grammar ())"));

        // "Header.ver" => [exp(["u8"])]
        assert!(s.contains("vec ! [pang :: nt (& < u8 as pang :: PangLabel > :: label ())]"));

        // rules.extend_grammar(u16::grammar());
        assert!(
            s.contains("rules . extend_grammar (& < u16 as pang :: ToGrammar > :: grammar ())")
        );

        // "Header.len" => [exp(["u16"])]
        assert!(s.contains("vec ! [pang :: nt (& < u16 as pang :: PangLabel > :: label ())]"));
    }
}
