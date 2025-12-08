//! Primary key `IndexedColumn` implementation generation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate `IndexedColumn` implementations for primary key columns.
pub fn generate_indexed_column_impls(
    table_name: &Ident,
    primary_key_columns: &[Ident],
) -> Vec<TokenStream> {
    let pk_column_types: Vec<_> = primary_key_columns
        .iter()
        .map(|col| quote! { #table_name::#col })
        .collect();

    primary_key_columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
            quote! {
                impl diesel_builders::IndexedColumn<
                    diesel_builders::typenum::#idx_type,
                    ( #(#pk_column_types,)* )
                > for #table_name::#col {}
            }
        })
        .collect()
}
