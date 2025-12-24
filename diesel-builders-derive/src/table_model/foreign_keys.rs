//! Generate foreign key implementations for triangular relations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident};

use crate::table_model::attribute_parsing::{
    extract_discretionary_table, extract_mandatory_table, extract_same_as_columns,
};

/// Generate foreign key implementations for triangular relations.
///
/// This function identifies columns with `#[mandatory(Table)]` or `#[discretionary(Table)]`
/// and pairs them with columns having `#[same_as(Table::Column)]` to generate
/// `HostColumn` implementations, effectively automating the `fk!` macro for these cases.
pub fn generate_foreign_key_impls(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
    table_module: &Ident,
) -> syn::Result<Vec<TokenStream>> {
    let mut impls = Vec::new();

    // 1. Identify mandatory/discretionary columns (M)
    for field in fields {
        let Some(field_name) = &field.ident else {
            continue;
        };
        let m_col = quote::quote!(#table_module::#field_name);

        // Check for mandatory or discretionary table reference
        let ref_table = if let Some(table) = extract_mandatory_table(field)? {
            table
        } else if let Some(table) = extract_discretionary_table(field)? {
            table
        } else {
            continue;
        };

        let ref_table_name = &ref_table
            .segments
            .last()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    &ref_table,
                    "Referenced table path must have at least one segment",
                )
            })?
            .ident;

        // 2. Find same_as columns (C) referencing the same table
        for other_field in fields {
            let Some(other_field_name) = &other_field.ident else {
                continue;
            };

            if field_name == other_field_name {
                continue;
            }

            for group in extract_same_as_columns(other_field)? {
                // Check for disambiguators in the group
                // A disambiguator is a path with a single segment that matches the current field name.
                // If there are any single-segment paths in the group, at least one must match `field_name`.
                let disambiguators: Vec<_> =
                    group.iter().filter(|p| p.segments.len() == 1).collect();

                if !disambiguators.is_empty() {
                    let matches_current_field = disambiguators
                        .iter()
                        .any(|p| p.segments.first().is_some_and(|s| s.ident == *field_name));

                    if !matches_current_field {
                        continue;
                    }
                }

                for ref_col in group {
                    // Check if path starts with ref_table
                    // We assume the path is like `RefTable::Column` or `Module1::Module2::RefTable::Column`
                    // So we check if the path excluding the last segment matches ref_table

                    let number_of_segments = ref_col.segments.len();
                    if number_of_segments < 2 {
                        continue;
                    }

                    let table_name = &ref_col.segments[number_of_segments - 2].ident;

                    // Construct a path from table_path_segments to compare with ref_table
                    // This is a bit heuristic. We check if ref_table ends with the table name found in same_as.
                    // Or better, we check if the segments match.

                    if ref_table_name == table_name {
                        // Use the fk! macro logic directly
                        impls.push(quote! {
                            diesel_builders::prelude::fk!(
                                (#m_col, #table_module::#other_field_name)
                                ->
                                (<<#m_col as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel::Table>::PrimaryKey, #ref_col)
                            );
                        });
                    }
                }
            }
        }
    }

    Ok(impls)
}
