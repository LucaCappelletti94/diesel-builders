//! `GetColumn` implementation generation for `TableModel` derive.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident, Token, punctuated::Punctuated};

/// Generate `GetColumn` trait implementations for all fields.
/// This replaces the separate `GetColumn` derive macro.
pub fn generate_get_column_impls(
    fields: &Punctuated<Field, Token![,]>,
    table_module: &syn::Ident,
    struct_ident: &Ident,
) -> TokenStream {
    let impls = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;
        Some(quote! {
            impl ::diesel_builders::GetColumn<#table_module::#field_name> for #struct_ident {
                fn get_column_ref(&self) -> &<#table_module::#field_name as ::diesel_builders::ColumnTyped>::ColumnType {
                    &self.#field_name
                }
            }
        })
    });

    quote! {
        #(#impls)*
    }
}
