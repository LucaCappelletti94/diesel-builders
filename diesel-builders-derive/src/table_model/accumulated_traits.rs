//! Logic for generating accumulated traits for `TableBuilder` and `TableModel`.
//!
//! This module generates two traits:
//! - `{StructName}TableBuilder`: Aggregates `SetColumn` and `TrySetColumn` for
//!   all columns.
//! - `{StructName}TableModel`: Aggregates `GetColumn` for all columns.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Field, Ident, Token, punctuated::Punctuated};

use super::attribute_parsing::is_field_infallible;

/// Generate `TableBuilder` and `TableModel` aggregated traits.
pub fn generate_accumulated_traits(
    fields: &Punctuated<Field, Token![,]>,
    table_module: &Ident,
    struct_ident: &Ident,
    primary_key_columns: &[Ident],
    is_surrogate_key: bool,
    has_error_type: bool,
) -> TokenStream {
    let table_builder_trait_ident =
        Ident::new(&format!("{struct_ident}TableBuilder"), Span::call_site());
    let table_model_trait_ident =
        Ident::new(&format!("{struct_ident}TableModel"), Span::call_site());

    let mut set_bounds = Vec::new();
    let mut get_bounds = Vec::new();

    for field in fields {
        if let Some(field_name) = &field.ident {
            let col_path = quote! { #table_module::#field_name };

            // Check if field is a primary key
            let is_pk = primary_key_columns.iter().any(|pk| pk == field_name);
            let is_surrogate = is_pk && is_surrogate_key;

            if !is_surrogate {
                let is_fallible = has_error_type && !is_field_infallible(field);
                if is_fallible {
                    set_bounds.push(quote! { ::diesel_builders::TrySetColumn<#col_path> });
                } else {
                    set_bounds.push(quote! { ::diesel_builders::SetColumn<#col_path> });
                }
            }

            get_bounds.push(quote! { ::diesel_builders::GetColumn<#col_path> });
        }
    }

    let table_builder_trait_doc = format!(
        "Aggregated trait ensuring a builder can set all columns for [`{struct_ident}`].\n\n\
         This trait aggregates [`SetColumn`](::diesel_builders::SetColumn) (or [`TrySetColumn`](::diesel_builders::TrySetColumn)) bounds for every column in the table\n\
         (excluding surrogate primary keys).\n\n\
         It is automatically implemented for any builder that satisfies these bounds."
    );

    let table_model_trait_doc = format!(
        "Aggregated trait ensuring access to all columns of [`{struct_ident}`].\n\n\
         This trait aggregates [`GetColumn`](::diesel_builders::GetColumn) bounds for every column in the table.\n\n\
         It is automatically implemented for any type (struct, generated model, or tuple)\n\
         that allows retrieving these columns."
    );

    let table_builder_trait = quote! {
        #[doc = #table_builder_trait_doc]
        pub trait #table_builder_trait_ident:
            #(#set_bounds)+*
        {}

        impl<T> #table_builder_trait_ident for T
        where
            T: #(#set_bounds)+*
        {}
    };

    let table_model_trait = quote! {
        #[doc = #table_model_trait_doc]
        pub trait #table_model_trait_ident:
            #(#get_bounds)+*
        {}

        impl<T> #table_model_trait_ident for T
        where
            T: #(#get_bounds)+*
        {}
    };

    quote! {
        #table_builder_trait
        #table_model_trait
    }
}
