//! Fuzz support for `diesel-builders-derive`.
//!
//! The derive modules are source-included from the production crate so the
//! targets exercise the exact parser and expansion code without duplicating
//! any algorithm or exposing a production fuzz API.

#[path = "../../diesel-builders-derive/src/descendant.rs"]
pub mod descendant;

#[path = "../../diesel-builders-derive/src/index.rs"]
pub mod index;

pub mod model_input;

#[path = "../../diesel-builders-derive/src/table_model.rs"]
pub mod table_model;

#[path = "../../diesel-builders-derive/src/utils.rs"]
pub mod utils;

/// Runs the `TableModel` derive expansion on a parsed struct.
pub fn derive_table_model(input: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    table_model::derive_table_model_impl(input)
}

/// Runs the shared `index!` / `unique_index!` expansion on a macro body.
pub fn expand_index(
    body: proc_macro2::TokenStream,
    trait_path: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    index::expand_index(body, trait_path)
}

/// Runs the owned field-attribute validation, returning its diagnostic.
pub fn validate_field_attributes(field: &syn::Field) -> syn::Result<()> {
    table_model::attribute_parsing::validate_field_attributes(field)
}

/// Extracts the referenced table from a `#[mandatory(table)]` field attribute.
pub fn extract_mandatory_table(field: &syn::Field) -> syn::Result<Option<syn::Path>> {
    table_model::attribute_parsing::extract_mandatory_table(field)
}

/// Extracts the referenced table from a `#[discretionary(table)]` field
/// attribute.
pub fn extract_discretionary_table(field: &syn::Field) -> syn::Result<Option<syn::Path>> {
    table_model::attribute_parsing::extract_discretionary_table(field)
}
