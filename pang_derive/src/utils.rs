use quote::quote;
use syn::{Attribute, DataStruct, Fields, Ident, Index, Lit, Type, parse_str};

/// BitField configuration
pub struct BitFieldConfig {
    pub label: String,
    pub start: usize,
    pub end: usize,
}

pub(crate) struct FieldGroup {
    /// Logical name
    ///
    /// - `#[pang(label = "c")]` => c;
    /// - Struct.field => field
    pub label: String,
    /// Corresponding physical field indices [0, 1]
    pub members: Vec<usize>,
    /// Whether it is a bitfield merge group
    pub is_bitfield: bool,
    /// Used to infer container type (u8/u16...)
    pub max_bits: usize,
    /// Origin type (Valid if not a bitfield)
    pub type_origin: Option<Type>,
    /// If type override is needed
    ///
    /// #[pang(semantic = "...")]
    pub type_override: Option<Ident>,
    /// If grammar override is needed
    ///
    /// #[pang(type="...")]
    pub grammar_override: Option<proc_macro2::TokenStream>,
}

/// Input DataStruct, Output FieldGroups
pub(crate) fn get_struct_groups(data: &DataStruct) -> Vec<FieldGroup> {
    let fields_processing = get_fields_processing_struct(data);
    let mut groups: Vec<FieldGroup> = Vec::new();

    for (idx, (_, ty, attrs, field_name_opt)) in fields_processing.iter().enumerate() {
        // Attempt to extract bitfield attributes (returns None if not a bitfield)
        let bitfield_info = get_pang_bitfield_attrs(attrs);
        let type_override = get_semantic_type_from_attrs(attrs);
        let grammar_override = get_type_override_from_attrs(attrs);

        if let Some(bf) = bitfield_info {
            // Bitfield processing

            // Check if the previous group exists, is a bitfield, and shares the same label
            let last_group_match = if let Some(last) = groups.last() {
                last.is_bitfield && last.label == bf.label
            } else {
                false
            };

            if last_group_match {
                // Merge into the previous group
                let group = groups.last_mut().unwrap();
                group.members.push(idx);
                // Update max_bits if the current field extends the bit range
                if bf.end > group.max_bits {
                    group.max_bits = bf.end;
                }
                if type_override.is_some() {
                    group.type_override = type_override;
                }
            } else {
                // Start a new group
                groups.push(FieldGroup {
                    label: bf.label,
                    members: vec![idx],
                    is_bitfield: true,
                    max_bits: bf.end,
                    type_origin: None, // Bitfield groups usually determine type by max_bits
                    type_override,
                    grammar_override,
                });
            }
        } else {
            // Normal field
            let label = get_field_label(field_name_opt, idx);
            groups.push(FieldGroup {
                label,
                members: vec![idx],
                is_bitfield: false,
                max_bits: 0,
                type_origin: Some((*ty).clone()), // Normal fields retain original type
                type_override,
                grammar_override,
            });
        }
    }
    groups
}

pub(crate) fn get_pang_bitfield_attrs(attrs: &[Attribute]) -> Option<BitFieldConfig> {
    let mut label = None;
    let mut bits_range = None;

    for attr in attrs {
        if attr.path().is_ident("pang") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("label") {
                    if let Lit::Str(lit) = meta.value()?.parse()? {
                        label = Some(lit.value());
                    }
                } else if meta.path.is_ident("bits") {
                    if let Lit::Str(lit) = meta.value()?.parse()? {
                        bits_range = Some(lit.value());
                    }
                }
                Ok(())
            });
        }
    }

    match (label, bits_range) {
        (Some(l), Some(range_str)) => {
            // Parse "0..4"
            let parts: Vec<&str> = range_str.split("..").collect();
            if parts.len() == 2 {
                let start = parts[0].parse::<usize>().ok()?;
                let end = parts[1].parse::<usize>().ok()?;
                Some(BitFieldConfig {
                    label: l,
                    start,
                    end,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

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

pub(crate) fn get_type_override_from_attrs(
    attrs: &[Attribute],
) -> Option<proc_macro2::TokenStream> {
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

    #[test]
    fn test_get_struct_groups() {
        let input: DeriveInput = parse_quote! {
            struct Packet {
                magic: u16,

                #[pang(label = "flags", bits = "0..4")]
                f1: u8,

                #[pang(label = "flags", bits = "4..7")]
                f2: u8,

                #[pang(label = "flags", bits = "7..8")]
                f3: u8,

                #[pang(semantic = "U32be")]
                payload: u32,
            }
        };

        let data = match input.data {
            Data::Struct(s) => s,
            _ => panic!("Should be struct"),
        };

        let groups = get_struct_groups(&data);

        assert_eq!(
            groups.len(),
            3,
            "Must have 3 groups (magic, flags, payload)"
        );

        let g1 = &groups[0];
        assert_eq!(g1.label, "magic");
        assert_eq!(g1.is_bitfield, false);
        assert_eq!(g1.members, vec![0]);
        assert!(g1.type_origin.is_some());

        let g2 = &groups[1];
        assert_eq!(g2.label, "flags");
        assert_eq!(g2.is_bitfield, true);
        assert_eq!(g2.members, vec![1, 2, 3]);
        assert_eq!(g2.max_bits, 8);
        assert!(
            g2.type_origin.is_none(),
            "Bitfield does not need type override"
        );

        let g3 = &groups[2];
        assert_eq!(g3.label, "payload");
        assert_eq!(g3.members, vec![4]);
    }
}
