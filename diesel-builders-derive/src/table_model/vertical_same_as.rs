//! Module for generating `VerticalSameAsGroup` implementations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use super::attribute_parsing::{TableModelAttributes, extract_same_as_columns};
use super::tokens_to_string;
use crate::utils::format_as_nested_tuple;

/// Generate `VerticalSameAsGroup` implementations for all columns in the table.
///
/// Each column will have an implementation with either:
/// - An empty tuple `()` if no `same_as` attributes are present
/// - A tuple of the specified columns if `same_as` attributes are present
pub fn generate_vertical_same_as_impls(
    fields: &Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
    attributes: &TableModelAttributes,
    triangular_tables: &std::collections::HashSet<syn::Path>,
) -> syn::Result<Vec<TokenStream>> {
    let mut impls = Vec::new();

    for field in fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;

        // Extract same_as columns from field attributes
        let same_as_attributes = extract_same_as_columns(field)?;
        let mut vertical_same_as_cols = Vec::new();

        for attr_paths in &same_as_attributes {
            // Check if this is a horizontal same_as with an explicit key
            // #[same_as(Target::Col, Key)]
            let is_horizontal_with_key = if attr_paths.len() == 2 {
                let first = &attr_paths[0];
                let second = &attr_paths[1];

                // Check if first is triangular
                let first_is_triangular = if first.segments.is_empty() {
                    false
                } else {
                    let table_name = &first.segments[0].ident;
                    triangular_tables.iter().any(|t| {
                        if let Some(segment) = t.segments.last() {
                            return segment.ident == *table_name;
                        }
                        false
                    })
                };

                // Check if second looks like a key (local table reference or single segment)
                let second_is_key = if second.segments.len() == 1 {
                    // Single segment (e.g. "key_field")
                    true
                } else if second.segments.len() >= 2 {
                    // Check if starts with table_module
                    second
                        .segments
                        .first()
                        .is_some_and(|s| s.ident == *table_module)
                } else {
                    false
                };

                first_is_triangular && second_is_key
            } else {
                false
            };

            for (i, column_path) in attr_paths.iter().enumerate() {
                // Skip the key if we identified this as a horizontal same_as with key
                if is_horizontal_with_key && i == 1 {
                    continue;
                }

                // Extract the table name from the column path (e.g., parent_table from parent_table::column)
                if column_path.segments.len() < 2 {
                    return Err(syn::Error::new_spanned(
                        column_path,
                        "Column path in #[same_as(...)] must be in the format `table::column`",
                    ));
                }

                let table_name = &column_path.segments[0].ident;

                // Check if this table is in the ancestors list
                let is_ancestor = if let Some(ancestors) = &attributes.ancestors {
                    ancestors.iter().any(|type_path| {
                        // Extract the identifier from the ancestor Type
                        if let Some(segment) = type_path.segments.last() {
                            return segment.ident == *table_name;
                        }
                        false
                    })
                } else {
                    false
                };

                // Check if this table is in the triangular tables list
                let is_triangular = triangular_tables.iter().any(|t| {
                    if let Some(segment) = t.segments.last() {
                        return segment.ident == *table_name;
                    }
                    false
                });

                if is_ancestor {
                    vertical_same_as_cols.push(column_path);
                } else if is_triangular {
                    // Ignore for VerticalSameAsGroup, handled by HorizontalKey
                } else {
                    return Err(syn::Error::new_spanned(
                        column_path,
                        format!(
                            "Column `{}` is not from an ancestor or triangular table. \
                             The #[same_as(...)] attribute can only reference columns from tables \
                             listed in #[table_model(ancestors(...))] or marked as #[mandatory]/#[discretionary]",
                            tokens_to_string(column_path)
                        ),
                    ));
                }
            }
        }

        if !vertical_same_as_cols.is_empty() && attributes.ancestors.is_none() {
            return Err(syn::Error::new_spanned(
                field,
                "Cannot use ancestor #[same_as(...)] on a table without ancestors. \
                 Add #[table_model(ancestors(...))] to specify the ancestor tables.",
            ));
        }

        let nested_columns = format_as_nested_tuple(vertical_same_as_cols);

        // Generate implementation for this column
        impls.push(quote! {
            impl diesel_builders::VerticalSameAsGroup for #table_module::#field_name {
                type VerticalSameAsNestedColumns = #nested_columns;
            }
        });
    }

    Ok(impls)
}
