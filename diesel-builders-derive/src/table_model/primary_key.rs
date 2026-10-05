//! Primary key `UniquelyIndexedColumn` implementation generation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate `UniquelyIndexedColumn` implementations for primary key columns.
pub fn generate_indexed_column_impls(
    table_module: &syn::Ident,
    primary_key_columns: &[Ident],
) -> Vec<TokenStream> {
    let columns: Vec<_> =
        primary_key_columns.iter().map(|col| quote! { #table_module::#col }).collect();
    crate::utils::index_impls(&quote! { ::diesel_builders::UniquelyIndexedColumn }, &columns)
}
