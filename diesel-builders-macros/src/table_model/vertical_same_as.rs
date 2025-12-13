//! Module for generating `VerticalSameAsGroup` implementations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

use crate::utils::format_as_nested_tuple;

use super::attribute_parsing::{extract_same_as_columns, TableModelAttributes};

/// Generate `VerticalSameAsGroup` implementations for all columns in the table.
///
/// Each column will have an implementation with either:
/// - An empty tuple `()` if no `same_as` attributes are present
/// - A tuple of the specified columns if `same_as` attributes are present
pub fn generate_vertical_same_as_impls(
    fields: &Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
    attributes: &TableModelAttributes,
) -> syn::Result<Vec<TokenStream>> {
    let mut impls = Vec::new();

    for field in fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;

        // Extract same_as columns from field attributes
        let same_as_columns = extract_same_as_columns(field)?;

        // Validate that all same_as columns are from ancestor tables
        if let Some(ancestors) = &attributes.ancestors {
            for column_path in &same_as_columns {
                // Extract the table name from the column path (e.g., parent_table from parent_table::column)
                if column_path.segments.len() < 2 {
                    return Err(syn::Error::new_spanned(
                        column_path,
                        "Column path in #[same_as(...)] must be in the format `table::column`",
                    ));
                }

                let table_name = &column_path.segments[0].ident;

                // Check if this table is in the ancestors list
                let is_ancestor = ancestors.iter().any(|ancestor| {
                    // Extract the identifier from the ancestor Type
                    if let syn::Type::Path(type_path) = ancestor {
                        if let Some(segment) = type_path.path.segments.last() {
                            return segment.ident == *table_name;
                        }
                    }
                    false
                });

                if !is_ancestor {
                    return Err(syn::Error::new_spanned(
                        column_path,
                        format!(
                            "Column `{}` is not from an ancestor table. \
                             The #[same_as(...)] attribute can only reference columns from tables \
                             listed in #[table_model(ancestors(...))]",
                            quote!(#column_path)
                        ),
                    ));
                }
            }
        } else if !same_as_columns.is_empty() {
            // If there are same_as columns but no ancestors defined
            return Err(syn::Error::new_spanned(
                field,
                "Cannot use #[same_as(...)] on a table without ancestors. \
                 Add #[table_model(ancestors(...))] to specify the ancestor tables.",
            ));
        }

        let nested_columns = format_as_nested_tuple(same_as_columns);

        // Generate implementation for this column
        impls.push(quote! {
            impl diesel_builders::VerticalSameAsGroup for #table_module::#field_name {
                type VerticalSameAsNestedColumns = #nested_columns;
            }
        });
    }

    Ok(impls)
}
