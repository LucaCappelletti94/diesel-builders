//! Module for `TableModel` derive macro implementation.
//!
//! This module contains the implementation of the `TableModel` derive macro,
//! split into logical components for better maintainability.

mod attribute_parsing;
mod get_column;
mod may_get_columns;
/// Module for generating `MaybeNullable` implementations.
mod maybe_nullable;
mod primary_key;
mod set_columns;
mod typed_column;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use attribute_parsing::{
    extract_field_default_value, extract_primary_key_columns, extract_table_model_attributes,
    extract_table_path, is_field_infallible, validate_field_attributes,
};
use get_column::generate_get_column_impls;
use maybe_nullable::generate_maybe_nullable_impls;
use primary_key::generate_indexed_column_impls;
use typed_column::generate_typed_column_impls;

use crate::utils::{format_as_nested_tuple, is_option};

/// Struct to hold processed field information.
struct ProcessedFields {
    /// Columns for the new record tuple.
    new_record_columns: Vec<syn::Path>,
    /// Records that can fail validation (index, path).
    fallible_records: Vec<(usize, syn::Path)>,
    /// Records that are infallible (index, path).
    infallible_records: Vec<(usize, syn::Path)>,
    /// Default values for fields.
    default_values: Vec<proc_macro2::TokenStream>,
}

/// Process fields to extract columns, validation status, and default values.
fn process_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_path: &syn::Path,
    primary_key_columns: &[syn::Ident],
    attributes: &attribute_parsing::TableModelAttributes,
) -> syn::Result<ProcessedFields> {
    let mut new_record_columns = Vec::new();
    let mut fallible_records = Vec::new();
    let mut infallible_records = Vec::new();
    let mut default_values = Vec::new();
    let mut idx = 0;

    for field in fields {
        validate_field_attributes(field)?;

        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;

        // Check if field is a primary key
        let is_pk = primary_key_columns.iter().any(|pk| pk == field_name);

        if is_pk {
            if extract_field_default_value(field).is_some() {
                return Err(syn::Error::new_spanned(
                    field,
                    "Primary key cannot have a default value",
                ));
            }

            if is_field_infallible(field) && attributes.surrogate_key {
                return Err(syn::Error::new_spanned(
                    field,
                    "Surrogate primary key cannot be marked as infallible",
                ));
            }
        }

        if is_pk && attributes.surrogate_key {
            continue;
        }

        new_record_columns.push(syn::parse_quote!(#table_path::#field_name));
        if is_field_infallible(field) || attributes.error.is_none() {
            infallible_records.push((idx, syn::parse_quote!(#table_path::#field_name)));
        } else {
            fallible_records.push((idx, syn::parse_quote!(#table_path::#field_name)));
        }

        // Default value logic
        let user_default = extract_field_default_value(field);
        let is_nullable = is_option(&field.ty);

        let default_val = if let Some(def) = user_default {
            quote::quote! { Some((#def).into()) }
        } else if is_nullable {
            quote::quote! { Some(None) }
        } else {
            quote::quote! { None }
        };
        default_values.push(default_val);

        idx += 1;
    }

    Ok(ProcessedFields {
        new_record_columns,
        fallible_records,
        infallible_records,
        default_values,
    })
}

/// Main entry point for the `TableModel` derive macro.
pub fn derive_table_model_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;

    // Parse attributes
    let table_path = extract_table_path(input)?;
    let primary_key_columns = extract_primary_key_columns(input);
    let attributes = extract_table_model_attributes(input);

    if attributes.surrogate_key && primary_key_columns.len() > 1 {
        return Err(syn::Error::new_spanned(
            input,
            "Surrogate key is not supported for composite primary keys",
        ));
    }

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
        generate_typed_column_impls(fields, &table_path, struct_ident, &primary_key_columns);
    let get_column_impls = generate_get_column_impls(fields, &table_path, struct_ident);
    let indexed_column_impls = generate_indexed_column_impls(&table_path, &primary_key_columns);
    let nested_primary_keys = format_as_nested_tuple(
        primary_key_columns
            .iter()
            .map(|col| quote::quote! { #table_path::#col }),
    );

    let ProcessedFields {
        new_record_columns,
        fallible_records,
        infallible_records,
        default_values,
    } = process_fields(fields, &table_path, &primary_key_columns, &attributes)?;

    let new_record = format_as_nested_tuple(&new_record_columns);
    let default_new_record = format_as_nested_tuple(&default_values);
    let new_record_type = format_as_nested_tuple(
        new_record_columns
            .iter()
            .map(|col| quote::quote! { Option<<#col as diesel_builders::Typed>::Type> }),
    );
    let may_get_column_impls =
        may_get_columns::generate_may_get_column_impls(&new_record_columns, &table_path);

    let set_column_impls = set_columns::generate_set_column_impls(&infallible_records, &table_path);

    let set_column_unchecked_impls =
        set_columns::generate_set_column_unchecked_traits(&fallible_records, &table_path);

    let maybe_nullable_impls = generate_maybe_nullable_impls(fields, &table_path);

    let error_type = attributes
        .error
        .map(|t| quote::quote! { #t })
        .unwrap_or(quote::quote! { std::convert::Infallible });

    // Generate final output
    Ok(quote! {
        #typed_column_impls
        #get_column_impls
        #(#indexed_column_impls)*
        #may_get_column_impls
        #set_column_impls
        #set_column_unchecked_impls
        #maybe_nullable_impls

        // Auto-implement TableExt for the table associated with this model.
        impl diesel_builders::TableExt for #table_path::table {
            type NewRecord = #new_record;
            type NewValues = #new_record_type;
            type Model = #struct_ident;
            type NestedPrimaryKeyColumns = #nested_primary_keys;
            type Error = #error_type;

            fn default_new_values() -> Self::NewValues {
                #default_new_record
            }
        }
    })
}
