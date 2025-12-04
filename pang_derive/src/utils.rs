use quote::quote;
use syn::{Attribute, DataStruct, Fields, Ident, Index, Lit, Type, parse_str};

pub(crate) fn get_fields_processing_struct(
    data: &DataStruct,
) -> Vec<(
    proc_macro2::TokenStream, // 1. Accessor: code to access the field (e.g., `self.name`)
    &Type,                    // 2. Type: the data type of the field (e.g., `i32`, `String`)
    &Vec<Attribute>, // 3. Attributes: macros on the field (e.g., `#[pang(semantic = "Payload")]`)
    Option<String>,  // 4. Name: field name as string (None for tuple structs)
)> {
    match &data.fields {
        // Named fields like `struct Header { ver: u8, len: u16 }`
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let name = &f.ident;
                (
                    quote! { self.#name }, // TokenStream: generates code `self.username`
                    &f.ty,
                    &f.attrs,
                    Some(name.as_ref().unwrap().to_string()), // Name as String: "username"
                )
            })
            .collect(), //
        // Unnamed fields like `struct Color(u8, u8, u8)`
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let index = Index::from(i);
                (
                    quote! { self.#index }, // TokenStream: generates code `self.0`, `self.1`
                    &f.ty,
                    &f.attrs,
                    None, // Tuple Structs don't have explicit names, so returns `None`
                )
            })
            .collect(),
        Fields::Unit => vec![],
    }
}

// Get field label by field name or index
pub(crate) fn get_field_label(name_opt: &Option<String>, index: usize) -> String {
    match name_opt {
        Some(name) => name.clone(),
        None => index.to_string(),
    }
}

/// Get semantic type from attributes
///
/// # Example
/// ```ignore
/// struct Packet {
///     header: u8,
///     #[pang(semantic = "Payload")] // <- semantic type
///     body: Vec<u8>
/// }
/// ```
pub(crate) fn get_semantic_type_from_attrs(attrs: &[Attribute]) -> Option<Ident> {
    for attr in attrs {
        if attr.path().is_ident("pang") {
            let mut res = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("semantic") {
                    if let Lit::Str(lit) = meta.value()?.parse()? {
                        res = Some(Ident::new(&lit.value(), lit.span()));
                    }
                }
                Ok(())
            });
            if res.is_some() {
                return res;
            }
        }
    }
    None
}

pub fn get_type_override_from_attrs(attrs: &[Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path().is_ident("pang") {
            let mut res = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    if let Lit::Str(lit) = meta.value()?.parse()? {
                        let expr_str = lit.value();
                        let expr: Type =
                            parse_str(&expr_str).expect("Invalid expression in pang(type=...)");
                        res = Some(quote! { #expr });
                    }
                }
                Ok(())
            });
            if res.is_some() {
                return res;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{Attribute, Data, DeriveInput, parse_quote};

    // -------------------------------------------------------------------------
    // 1. Attribute tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_semantic_type() {
        // Case A: multiple attributes
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[derive(Debug)]),                // Distractor
            parse_quote!(#[pang(semantic = "MyWrapper")]), // Target
        ];

        let result = get_semantic_type_from_attrs(&attrs);
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "MyWrapper");
    }

    #[test]
    fn test_get_semantic_type_none() {
        // Case B: No semantic
        let attrs: Vec<Attribute> = vec![parse_quote!(#[derive(Debug)])];
        assert!(get_semantic_type_from_attrs(&attrs).is_none());

        // Case C: Invalid semantic
        let attrs_invalid: Vec<Attribute> = vec![parse_quote!(#[pang(other = "val")])];
        assert!(get_semantic_type_from_attrs(&attrs_invalid).is_none());
    }

    #[test]
    fn test_get_type_override() {
        // User force type
        let attrs: Vec<Attribute> = vec![parse_quote!(#[pang(type = "Box < u8 >")])];

        let result = get_type_override_from_attrs(&attrs);
        assert!(result.is_some());

        assert_eq!(result.unwrap().to_string(), "Box < u8 >");
    }

    // -------------------------------------------------------------------------
    // 2. Label tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_field_label_naming() {
        // Named field: field name
        assert_eq!(get_field_label(&Some("ver".to_string()), 0), "ver");

        // Unnamed field (Tuple): index
        assert_eq!(get_field_label(&None, 5), "5");
    }

    // -------------------------------------------------------------------------
    // 3. Test filed in struct
    // -------------------------------------------------------------------------

    #[test]
    fn test_process_struct_named_fields() {
        let input: DeriveInput = parse_quote! {
            struct Packet {
                header: u8,
                #[pang(semantic = "Payload")]
                body: Vec<u8>
            }
        };

        let data = match input.data {
            Data::Struct(s) => s,
            _ => panic!("Should be struct"),
        };

        let results = get_fields_processing_struct(&data);

        assert_eq!(results.len(), 2);

        // Check header
        let (acc1, _ty1, _attrs1, name1) = &results[0];
        assert_eq!(acc1.to_string(), "self . header");
        assert_eq!(name1.as_ref().unwrap(), "header");

        // Check body
        let (acc2, _ty2, attrs2, name2) = &results[1];
        assert_eq!(acc2.to_string(), "self . body");
        assert_eq!(name2.as_ref().unwrap(), "body");

        assert!(get_semantic_type_from_attrs(attrs2).is_some());
    }

    #[test]
    fn test_process_struct_tuple_fields() {
        // Test Tuple Struct
        let input: DeriveInput = parse_quote! {
            struct Color(u8, u8);
        };

        let data = match input.data {
            Data::Struct(s) => s,
            _ => panic!("Should be struct"),
        };

        let results = get_fields_processing_struct(&data);

        assert_eq!(results.len(), 2);

        let (acc1, _, _, name1) = &results[0];
        assert_eq!(acc1.to_string(), "self . 0");
        assert!(name1.is_none());

        let (acc2, _, _, _) = &results[1];
        assert_eq!(acc2.to_string(), "self . 1");
    }
}
