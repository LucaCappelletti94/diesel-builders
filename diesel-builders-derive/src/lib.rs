#![doc = include_str!("../README.md")]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::allow_attributes,
        clippy::allow_attributes_without_reason,
        clippy::fallible_impl_from,
    )
)]

mod descendant;
mod index;
mod table_model;
mod utils;
use proc_macro::TokenStream;
/// Generates a Diesel table, checked builders, and relationship implementations
/// for a model.
#[proc_macro_derive(
    TableModel,
    attributes(table_model, infallible, mandatory, discretionary, diesel, same_as)
)]
pub fn derive_table_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match table_model::derive_table_model_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Converts native index expansion into compiler tokens or diagnostics.
fn generate_index_impl(input: TokenStream, trait_path: &proc_macro2::TokenStream) -> TokenStream {
    match index::expand_index(input.into(), trait_path) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Generates `UniquelyIndexedColumn` implementations for each indexed column.
#[proc_macro]
pub fn unique_index(input: TokenStream) -> TokenStream {
    generate_index_impl(input, &quote::quote!(::diesel_builders::UniquelyIndexedColumn))
}

/// Generates `IndexedColumn` implementations for each indexed column.
#[proc_macro]
pub fn index(input: TokenStream) -> TokenStream {
    generate_index_impl(input, &quote::quote!(::diesel_builders::IndexedColumn))
}
