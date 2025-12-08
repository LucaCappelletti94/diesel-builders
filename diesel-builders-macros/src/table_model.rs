//! Module for `TableModel` derive macro implementation.
//!
//! This module contains the implementation of the `TableModel` derive macro,
//! split into logical components for better maintainability.

mod attribute_parsing;
mod get_column;
mod primary_key;
mod typed_column;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use attribute_parsing::{extract_insertable_name, extract_primary_key_columns, extract_table_name};
use get_column::generate_get_column_impls;
use primary_key::generate_indexed_column_impls;
use typed_column::generate_typed_column_impls;

/// Main entry point for the `TableModel` derive macro.
pub fn derive_table_model_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;

    // Parse attributes
    let table_name = extract_table_name(input)?;
    let primary_key_columns = extract_primary_key_columns(input);
    let insertable_ident = extract_insertable_name(input, struct_ident);

    // Extract fields
    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "TableModel can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "TableModel can only be derived for structs",
            ));
        }
    };

    // Generate all components
    let typed_column_impls =
        generate_typed_column_impls(fields, &table_name, struct_ident, &primary_key_columns);
    let get_column_impls = generate_get_column_impls(fields, &table_name, struct_ident);
    let indexed_column_impls = generate_indexed_column_impls(&table_name, &primary_key_columns);

    // Generate final output
    Ok(quote! {
        #typed_column_impls
        #get_column_impls
        #(#indexed_column_impls)*

        // Auto-implement TableAddition for the table associated with this model.
        impl diesel_builders::TableAddition for #table_name::table {
            type InsertableModel = #insertable_ident;
            type Model = #struct_ident;
            type PrimaryKeyColumns = (#(#table_name::#primary_key_columns,)*) ;
        }
    })
}
