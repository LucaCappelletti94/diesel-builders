//! Generate foreign key implementations for triangular relations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident};

use crate::table_model::attribute_parsing::{
    ForeignKeyAttribute, ForeignKeyTarget, extract_discretionary_table, extract_mandatory_table,
    extract_same_as_columns,
};

/// Generate foreign key implementations for triangular relations.
///
/// This function identifies columns with `#[mandatory(Table)]` or
/// `#[discretionary(Table)]` and pairs them with columns having
/// `#[same_as(Table::Column)]` to generate `HostColumn` implementations,
/// effectively automating the `fk!` macro for these cases.
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
                // A disambiguator is a path with a single segment that matches the current
                // field name. If there are any single-segment paths in the
                // group, at least one must match `field_name`.
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
                    // We assume the path is like `RefTable::Column` or
                    // `Module1::Module2::RefTable::Column` So we check if the
                    // path excluding the last segment matches ref_table

                    let number_of_segments = ref_col.segments.len();
                    if number_of_segments < 2 {
                        continue;
                    }

                    let table_name = &ref_col.segments[number_of_segments - 2].ident;

                    // Construct a path from table_path_segments to compare with ref_table
                    // This is a bit heuristic. We check if ref_table ends with the table name found
                    // in same_as. Or better, we check if the segments match.

                    if ref_table_name == table_name {
                        // Generate HostColumn implementations directly
                        let host_cols = quote! { #m_col, #table_module::#other_field_name };
                        let ref_cols = quote! {
                            <<#m_col as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel::Table>::PrimaryKey,
                            #ref_col
                        };

                        // 1. Generate allow_tables_to_appear_in_same_query
                        // We use the second column for table extraction as the first one is complex
                        // (PrimaryKey)
                        if let Some(ref_table) =
                            crate::utils::extract_table_path_from_column(&ref_col)
                        {
                            let host_table: syn::Path = syn::parse_quote!(#table_module);
                            if crate::utils::should_generate_allow_tables_to_appear_in_same_query(
                                &host_table,
                                &ref_table,
                            ) {
                                impls.push(quote! {
                                    ::diesel::allow_tables_to_appear_in_same_query!(#host_table, #ref_table);
                                });
                            }
                        }

                        // 2. Impl HostColumn for col 0
                        impls.push(quote! {
                            impl diesel_builders::HostColumn<
                                diesel_builders::typenum::U0,
                                ( #host_cols ),
                                ( #ref_cols )
                            > for #m_col {}
                        });

                        // 3. Impl HostColumn for col 1
                        impls.push(quote! {
                            impl diesel_builders::HostColumn<
                                diesel_builders::typenum::U1,
                                ( #host_cols ),
                                ( #ref_cols )
                            > for #table_module::#other_field_name {}
                        });
                    }
                }
            }
        }
    }

    Ok(impls)
}

/// Generate explicit foreign key and foreign primary key implementations from
/// `#[table_model(foreign_key)]` attributes.
pub fn generate_explicit_foreign_key_impls(
    foreign_keys: &[ForeignKeyAttribute],
    table_module: &Ident,
) -> syn::Result<Vec<TokenStream>> {
    let mut impls = Vec::new();
    let host_table_path: syn::Path = syn::parse_quote!(#table_module);

    for fk in foreign_keys {
        match &fk.target {
            ForeignKeyTarget::Table(table_path) => {
                if fk.host_columns.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        table_path,
                        "Foreign Primary Key (FPK) syntax requires exactly one host column.",
                    ));
                }
                let host_col_ident = &fk.host_columns[0];
                let column_path: syn::Path = syn::parse_quote!(#table_module::#host_col_ident);

                // FPK implies allow_tables_to_appear_in_same_query
                if crate::utils::should_generate_allow_tables_to_appear_in_same_query(
                    &host_table_path,
                    table_path,
                ) {
                    impls.push(quote! { ::diesel::allow_tables_to_appear_in_same_query!(#table_module, #table_path); });
                }

                // Generate FPK
                if let Some(stream) = generate_fpk_impl(&column_path, table_path) {
                    impls.push(stream);
                }
            }
            ForeignKeyTarget::Columns(ref_cols) => {
                if fk.host_columns.len() != ref_cols.len() {
                    return Err(syn::Error::new_spanned(
                        &ref_cols[0],
                        "Mismatched number of host and referenced columns.",
                    ));
                }

                // Try extract table from first ref col for allow_same_query
                if let Some(first_ref) = ref_cols.first()
                    && let Some(ref_table) = crate::utils::extract_table_path_from_column(first_ref)
                    && crate::utils::should_generate_allow_tables_to_appear_in_same_query(
                        &host_table_path,
                        &ref_table,
                    )
                {
                    impls.push(
                        quote! { ::diesel::allow_tables_to_appear_in_same_query!(#table_module, #ref_table); },
                    );
                }

                let host_cols_tokens: Vec<_> =
                    fk.host_columns.iter().map(|c| quote!(#table_module::#c)).collect();

                for (idx, host_col_ident) in fk.host_columns.iter().enumerate() {
                    let idx_type =
                        syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
                    let host_col = quote!(#table_module::#host_col_ident);
                    impls.push(quote! {
                        impl ::diesel_builders::HostColumn<
                            ::diesel_builders::typenum::#idx_type,
                            ( #(#host_cols_tokens,)* ),
                            ( #(#ref_cols,)* )
                        > for #host_col {}
                    });
                }
            }
        }
    }

    Ok(impls)
}

/// Generate a foreign primary key implementation for a column.
///
/// This function generates:
/// 1. `ForeignPrimaryKey` implementation for the column
/// 2. A helper trait with a method to fetch the foreign record
///
/// # Arguments
/// * `column` - The column path (e.g., `table_b::c_id`)
/// * `referenced_table` - The referenced table type (e.g., `table_c`)
pub fn generate_fpk_impl(column: &syn::Path, referenced_table: &syn::Path) -> Option<TokenStream> {
    // Extract column name for method generation
    let last_segment = column.segments.last()?;
    let column_name = last_segment.ident.to_string();

    // Extract referenced table name for method generation
    let last_segment = referenced_table.segments.last()?;
    let referenced_table_name = last_segment.ident.to_string();

    // Generate method name based on column name
    let method_name = if let Some(stripped) = column_name.strip_suffix("_id") {
        stripped.to_string()
    } else {
        format!("{column_name}_fk")
    };
    let method_ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

    // Generate trait name
    // Extract table name from column path (second-to-last segment)
    assert!(
        column.segments.len() >= 2,
        "Column path must have at least 2 segments (table::column)"
    );
    let table_name_segment = column.segments[column.segments.len() - 2].ident.to_string();

    // Convert table_name to CamelCase for trait name
    let trait_name = format!(
        "FK{}{}",
        crate::utils::snake_to_camel_case(&table_name_segment),
        crate::utils::snake_to_camel_case(&column_name)
    );
    let trait_ident = syn::Ident::new(&trait_name, proc_macro2::Span::call_site());

    // Generate documentation
    let trait_doc = format!("Trait to get the foreign record referenced by `{column_name}`.");
    let method_doc = format!(
        "Fetches the foreign `{referenced_table_name}` record referenced by this `{column_name}`."
    );

    Some(quote! {
        impl ::diesel_builders::ForeignPrimaryKey for #column {
            type ReferencedTable = #referenced_table::table;
        }

        #[doc = #trait_doc]
        pub trait #trait_ident<Conn>: ::diesel_builders::GetForeign<
            Conn,
            (#column,),
            (<#referenced_table::table as ::diesel::Table>::PrimaryKey,),
        > {
            #[doc = #method_doc]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = ""]
            #[doc = "* `conn` - A mutable reference to the database connection."]
            #[doc = ""]
            #[doc = "# Errors"]
            #[doc = "Returns a `diesel::QueryResult` error if the query fails or no matching record is found."]
            #[inline]
            fn #method_ident(
                &self,
                conn: &mut Conn,
            ) -> ::diesel::QueryResult<<#referenced_table::table as ::diesel_builders::TableExt>::Model>
            {
                <Self as ::diesel_builders::GetForeign<
                    Conn,
                    (#column,),
                    (<#referenced_table::table as ::diesel::Table>::PrimaryKey,),
                >>::foreign(self, conn)
            }
        }

        impl<T, Conn> #trait_ident<Conn> for T
        where
            T: ::diesel_builders::GetForeign<
                Conn,
                (#column,),
                (<#referenced_table::table as ::diesel::Table>::PrimaryKey,)
            > {}
    })
}
