//! Generates the `diesel::table!` macro for a given struct representing a table model.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Field, Ident, Path, Type};

use crate::utils::is_option;

/// Infers the Diesel SQL type from a Rust type.
fn infer_sql_type(ty: &Type) -> Option<TokenStream> {
    if is_option(ty) {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        let inner_sql_type = infer_sql_type(inner_ty)?;
                        return Some(quote! { diesel::sql_types::Nullable<#inner_sql_type> });
                    }
                }
            }
        }
        return None;
    }

    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            if type_name == "Vec" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        // Check if inner_ty is u8 for Binary
                        if let Type::Path(inner_path) = inner_ty {
                            if let Some(inner_segment) = inner_path.path.segments.last() {
                                if inner_segment.ident == "u8" {
                                    return Some(quote! { diesel::sql_types::Binary });
                                }
                            }
                        }

                        // Otherwise infer for Array<T>
                        if let Some(inner_sql_type) = infer_sql_type(inner_ty) {
                            return Some(quote! { diesel::sql_types::Array<#inner_sql_type> });
                        }
                    }
                }
            }

            match type_name.as_str() {
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
        } else {
            None
        }
    } else {
        None
    }
}

/// Extracts the SQL type from the `#[diesel(sql_type = ...)]` attribute or infers it.
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
    table_path: &Path,
    primary_key_columns: &[Ident],
) -> syn::Result<TokenStream> {
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

    let mut column_defs = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let sql_type = get_column_sql_type(field)?;

        // Preserve documentation
        let doc_attrs = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"));

        column_defs.push(quote! {
            #(#doc_attrs)*
            #field_name -> #sql_type,
        });
    }

    let struct_doc_attrs = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"));

    let pk_tuple = if primary_key_columns.len() == 1 {
        let pk = &primary_key_columns[0];
        quote! { #pk }
    } else {
        quote! { #(#primary_key_columns),* }
    };

    // Use the last segment of the path as the table name for definition
    let table_name_ident = &table_path.segments.last().unwrap().ident;

    Ok(quote! {
        diesel::table! {
            #(#struct_doc_attrs)*
            #table_name_ident (#pk_tuple) {
                #(#column_defs)*
            }
        }
    })
}
