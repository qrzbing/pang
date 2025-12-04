use quote::quote;
use syn::DeriveInput;

pub fn derive_pang_label_impl(input: DeriveInput) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    // TODO: support #[pang(label="CustomName")]
    let struct_label = struct_name.to_string();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics pang::PangLabel for #struct_name #ty_generics #where_clause {
            fn label() -> String {
                #struct_label.to_string()
            }
        }
    }
}
