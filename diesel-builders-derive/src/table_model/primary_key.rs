//! Primary key `UniquelyIndexedColumn` implementation generation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident};

/// Generate `UniquelyIndexedColumn` implementations for primary key columns.
pub fn generate_indexed_column_impls(
    table_module: &syn::Ident,
    struct_ident: &syn::Ident,
    primary_key_columns: &[Ident],
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> Vec<TokenStream> {
    let pk_column_types: Vec<_> = primary_key_columns
        .iter()
        .map(|col| quote! { #table_module::#col })
        .collect();

    let mut impls: Vec<TokenStream> = primary_key_columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
            quote! {
                impl diesel_builders::UniquelyIndexedColumn<
                    diesel_builders::typenum::#idx_type,
                    ( #(#pk_column_types,)* )
                > for #table_module::#col {}
            }
        })
        .collect();

    // From implementations
    let pk_fields: Vec<_> = primary_key_columns
        .iter()
        .map(|pk_col| {
            fields
                .iter()
                .find(|f| f.ident.as_ref() == Some(pk_col))
                .expect("Primary key column should exist")
        })
        .collect();

    let pk_types: Vec<_> = pk_fields.iter().map(|f| &f.ty).collect();
    let pk_names: Vec<_> = pk_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    let target_type = if pk_types.len() == 1 {
        let ty = pk_types[0];
        quote! { #ty }
    } else {
        quote! { (#(#pk_types),*) }
    };

    let body_ref = if pk_names.len() == 1 {
        let name = pk_names[0];
        quote! { model.#name.clone() }
    } else {
        let names = &pk_names;
        quote! { ( #(model.#names.clone()),* ) }
    };

    let body_val = if pk_names.len() == 1 {
        let name = pk_names[0];
        quote! { model.#name }
    } else {
        let names = &pk_names;
        quote! { ( #(model.#names),* ) }
    };

    impls.push(quote! {
        impl From<&#struct_ident> for #target_type {
            fn from(model: &#struct_ident) -> Self {
                #body_ref
            }
        }
        impl From<#struct_ident> for #target_type {
            fn from(model: #struct_ident) -> Self {
                #body_val
            }
        }
    });

    impls
}
