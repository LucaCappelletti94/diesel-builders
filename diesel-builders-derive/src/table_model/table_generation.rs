//! Generates the `diesel::table!` macro for a given struct representing a table
//! model.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, Ident, Type};

use crate::{table_model::attribute_parsing::extract_sql_name, utils::is_option};

/// Extracts the first generic type argument from a type path, if it exists.
fn extract_first_generic_arg(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let syn::PathArguments::AngleBracketed(args) = &type_path.path.segments.last()?.arguments
        && let syn::GenericArgument::Type(inner_ty) = args.args.first()?
    {
        Some(inner_ty)
    } else {
        None
    }
}

/// Maps primitive Rust types to their corresponding Diesel SQL types.
fn map_primitive_type(type_name: &str) -> Option<TokenStream> {
    match type_name {
        "i32" => Some(quote! { diesel::sql_types::Integer }),
        "i64" => Some(quote! { diesel::sql_types::BigInt }),
        "i16" => Some(quote! { diesel::sql_types::SmallInt }),
        "f32" => Some(quote! { diesel::sql_types::Float }),
        "f64" => Some(quote! { diesel::sql_types::Double }),
        "bool" => Some(quote! { diesel::sql_types::Bool }),
        "String" => Some(quote! { diesel::sql_types::Text }),
        "NaiveDate" => Some(quote! { diesel::sql_types::Date }),
        "NaiveDateTime" => Some(quote! { diesel::sql_types::Timestamp }),
        "NaiveTime" => Some(quote! { diesel::sql_types::Time }),
        "Uuid" => Some(quote! { diesel::sql_types::Uuid }),
        _ => None,
    }
}

/// Infers the Diesel SQL type from a Rust type.
fn infer_sql_type(ty: &Type) -> Option<TokenStream> {
    // Handle Option<T> -> Nullable<InnerType>
    if is_option(ty) {
        let inner_ty = extract_first_generic_arg(ty)?;
        let inner_sql_type = infer_sql_type(inner_ty)?;
        return Some(quote! { diesel::sql_types::Nullable<#inner_sql_type> });
    }

    // Handle other types
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last()?;
        let type_name = segment.ident.to_string();

        // Handle Vec<T>
        if type_name == "Vec" {
            let inner_ty = extract_first_generic_arg(ty)?;

            // Special case: Vec<u8> -> Binary
            if let Type::Path(inner_path) = inner_ty
                && let Some(inner_segment) = inner_path.path.segments.last()
                && inner_segment.ident == "u8"
            {
                return Some(quote! { diesel::sql_types::Binary });
            }

            // General case: Vec<T> -> Array<InnerType>
            if let Some(inner_sql_type) = infer_sql_type(inner_ty) {
                return Some(quote! { diesel::sql_types::Array<#inner_sql_type> });
            }
        }

        // Handle primitive types
        map_primitive_type(&type_name)
    } else {
        None
    }
}

/// Extracts the SQL type from the `#[diesel(sql_type = ...)]` attribute or
/// infers it.
fn get_column_sql_type(field: &Field) -> syn::Result<TokenStream> {
    let mut found_sql_type = None;

    // Check for #[diesel(sql_type = ...)] attribute
    for attr in &field.attrs {
        if attr.path().is_ident("diesel") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("sql_type") {
                    if found_sql_type.is_some() {
                        return Err(meta.error("Duplicate sql_type attribute"));
                    }
                    let value = meta.value()?;
                    let type_path: syn::Path = value.parse()?;
                    found_sql_type = Some(quote! { #type_path });
                }
                Ok(())
            })?;
        }
    }

    if let Some(sql_type) = found_sql_type {
        // If the field is an Option<T>, we should wrap the provided SQL type in
        // Nullable<...>. This assumes the user provided the "inner" SQL type in
        // the attribute.
        if is_option(&field.ty) {
            return Ok(quote! { ::diesel::sql_types::Nullable<#sql_type> });
        }
        return Ok(sql_type);
    }

    // Try to infer
    if let Some(sql_type) = infer_sql_type(&field.ty) {
        Ok(sql_type)
    } else {
        Err(syn::Error::new_spanned(
            field,
            "Could not infer SQL type. Please specify it using #[diesel(sql_type = ...)]",
        ))
    }
}

/// Generates the `diesel::table!` macro call.
pub fn generate_table_macro(
    input: &DeriveInput,
    table_module: &Ident,
    primary_key_columns: &[Ident],
) -> syn::Result<TokenStream> {
    let fields = match &input.data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => {
                    return Err(syn::Error::new_spanned(
                        input,
                        "TableModel can only be derived for structs with named fields",
                    ));
                }
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "TableModel can only be derived for structs",
            ));
        }
    };

    let mut column_defs = Vec::new();

    for field in fields {
        let Some(field_name) = &field.ident else {
            continue;
        };
        let sql_type = get_column_sql_type(field)?;

        // Preserve documentation
        let doc_attrs = field.attrs.iter().filter(|attr| attr.path().is_ident("doc"));

        let sql_name_attr = extract_sql_name(field).map(|name| {
            quote! { #[sql_name = #name] }
        });

        column_defs.push(quote! {
            #(#doc_attrs)*
            #sql_name_attr
            #field_name -> #sql_type,
        });
    }

    let struct_doc_attrs = input.attrs.iter().filter(|attr| attr.path().is_ident("doc"));

    // Use the module identifier as the table name for definition
    Ok(quote! {
        diesel::table! {
            #(#struct_doc_attrs)*
            #table_module (#(#primary_key_columns),*) {
                #(#column_defs)*
            }
        }
    })
}
