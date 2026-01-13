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
                        let host_cols =
                            quote! { #table_module::#field_name, #table_module::#other_field_name };
                        let ref_cols = quote! {
                            <<#table_module::#field_name as diesel_builders::ForeignPrimaryKey>::ReferencedTable as diesel::Table>::PrimaryKey,
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
                            > for #table_module::#field_name {}
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

                // FPK implies allow_tables_to_appear_in_same_query
                if crate::utils::should_generate_allow_tables_to_appear_in_same_query(
                    &host_table_path,
                    table_path,
                ) {
                    impls.push(quote! { ::diesel::allow_tables_to_appear_in_same_query!(#table_module, #table_path); });
                }

                // Generate FPK
                if let Some(stream) = generate_fpk_impl(
                    &syn::parse_quote!(#table_module::#host_col_ident),
                    table_path,
                ) {
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
/// Metadata for a captured foreign key relationship used in `IterForeignKey`
/// generation.
struct CapturedForeignKey {
    /// Host table field identifiers forming the foreign key
    host_fields: Vec<syn::Ident>,
    /// Referenced column paths in the target table
    ref_cols: Vec<TokenStream>,
    /// Whether each host field should be unwrapped if it's an `Option`
    unwrap_if_option: Vec<bool>,
    /// Unique key for grouping foreign keys that reference the same index
    grouping_key: String,
}

/// Generates implementations of `IterForeignKey` for the table model.
///
/// This function analyzes both implicit (triangular relations via
/// `#[mandatory]`/`#[discretionary]`) and explicit
/// (`#[table_model(foreign_key)]`) foreign keys, groups them by their
/// referenced unique index, and generates `IterForeignKey` trait
/// implementations for each group.
///
/// The generated iterators yield flat tuples of references to the foreign key
/// values, automatically handling `Option` types by filtering out `None`
/// values.
pub fn generate_iter_foreign_key_impls(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
    foreign_keys: &[ForeignKeyAttribute],
    table_module: &Ident,
    model_ident: &Ident,
) -> syn::Result<Vec<TokenStream>> {
    let captured_keys = collect_foreign_keys(fields, foreign_keys, table_module)?;

    // Group foreign keys by their referenced index
    let groups = group_by_referenced_index(captured_keys);

    // Generate an IterForeignKey impl for each unique referenced index
    Ok(generate_impls_for_groups(groups, fields, model_ident))
}

/// Collects all foreign key relationships (both implicit and explicit).
fn collect_foreign_keys(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
    foreign_keys: &[ForeignKeyAttribute],
    table_module: &Ident,
) -> syn::Result<Vec<CapturedForeignKey>> {
    let mut captured_keys = Vec::new();

    // Collect implicit foreign keys from triangular relations
    collect_triangular_foreign_keys(fields, table_module, &mut captured_keys)?;

    // Collect explicit foreign keys from attributes
    collect_explicit_foreign_keys(foreign_keys, table_module, &mut captured_keys);

    Ok(captured_keys)
}

/// Collects foreign keys from mandatory/discretionary triangular relations.
fn collect_triangular_foreign_keys(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
    _table_module: &Ident,
    captured_keys: &mut Vec<CapturedForeignKey>,
) -> syn::Result<()> {
    for field in fields {
        let Some(field_name) = &field.ident else {
            continue;
        };

        // Check for mandatory/discretionary table reference
        let ref_table = if let Some(table) = extract_mandatory_table(field)? {
            table
        } else if let Some(table) = extract_discretionary_table(field)? {
            table
        } else {
            continue;
        };

        let ref_table_name = &ref_table.segments.last().unwrap().ident;

        // Find same_as columns that reference this table
        for other_field in fields {
            let Some(other_field_name) = &other_field.ident else {
                continue;
            };

            if field_name == other_field_name {
                continue;
            }

            for group in extract_same_as_columns(other_field)? {
                // Check disambiguators: if there are single-segment paths in the group,
                // at least one must match the current field name
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
                    if ref_col.segments.len() < 2 {
                        continue;
                    }

                    let table_name = &ref_col.segments[ref_col.segments.len() - 2].ident;

                    if ref_table_name == table_name {
                        // Found a triangular FK: (mandatory/discr_id, same_as_field) ->
                        // (RefTable::PK, RefTable::column)
                        let ref_pk = quote!(
                            <#ref_table::table as ::diesel::Table>::PrimaryKey
                        );

                        let col_name = &ref_col.segments.last().unwrap().ident;
                        let grouping_key = format!("{ref_table_name}::{col_name}");

                        captured_keys.push(CapturedForeignKey {
                            host_fields: vec![field_name.clone(), other_field_name.clone()],
                            ref_cols: vec![ref_pk, quote!(#ref_col)],
                            // Both columns must be unwrapped if Option for triangular relations
                            unwrap_if_option: vec![true, true],
                            grouping_key,
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

/// Collects foreign keys from explicit `#[table_model(foreign_key)]`
/// attributes.
fn collect_explicit_foreign_keys(
    foreign_keys: &[ForeignKeyAttribute],
    _table_module: &Ident,
    captured_keys: &mut Vec<CapturedForeignKey>,
) {
    for fk in foreign_keys {
        match &fk.target {
            ForeignKeyTarget::Table(table_path) => {
                // Foreign Primary Key: single column -> table's primary key
                if fk.host_columns.len() == 1 {
                    let host_col = &fk.host_columns[0];
                    let ref_pk = quote!(
                        <#table_path::table as ::diesel::Table>::PrimaryKey
                    );

                    let table_name = &table_path.segments.last().unwrap().ident;
                    let grouping_key = format!("{table_name}::PrimaryKey");

                    captured_keys.push(CapturedForeignKey {
                        host_fields: vec![host_col.clone()],
                        ref_cols: vec![ref_pk],
                        unwrap_if_option: vec![true],
                        grouping_key,
                    });
                }
            }
            ForeignKeyTarget::Columns(ref_cols_paths) => {
                // Explicit multi-column foreign key
                if fk.host_columns.len() == ref_cols_paths.len() {
                    let ref_cols_tokens = ref_cols_paths.iter().map(|p| quote!(#p)).collect();

                    // Create a unique grouping key from the referenced columns
                    let parts: Vec<String> = ref_cols_paths
                        .iter()
                        .map(|p| {
                            if p.segments.len() >= 2 {
                                let table = &p.segments[p.segments.len() - 2].ident;
                                let column = &p.segments.last().unwrap().ident;
                                format!("{table}::{column}")
                            } else {
                                p.segments.last().unwrap().ident.to_string()
                            }
                        })
                        .collect();
                    let grouping_key = parts.join(", ");

                    captured_keys.push(CapturedForeignKey {
                        host_fields: fk.host_columns.clone(),
                        ref_cols: ref_cols_tokens,
                        unwrap_if_option: vec![true; fk.host_columns.len()],
                        grouping_key,
                    });
                }
            }
        }
    }
}

/// Groups foreign keys by their referenced index.
fn group_by_referenced_index(
    captured_keys: Vec<CapturedForeignKey>,
) -> std::collections::HashMap<String, (Vec<TokenStream>, Vec<CapturedForeignKey>)> {
    use std::collections::HashMap;

    let mut groups: HashMap<String, (Vec<TokenStream>, Vec<CapturedForeignKey>)> = HashMap::new();

    for key in captured_keys {
        groups
            .entry(key.grouping_key.clone())
            .or_insert_with(|| (key.ref_cols.clone(), Vec::new()))
            .1
            .push(key);
    }

    groups
}

/// Generates `IterForeignKey` trait implementations for each group of foreign
/// keys.
fn generate_impls_for_groups(
    groups: std::collections::HashMap<String, (Vec<TokenStream>, Vec<CapturedForeignKey>)>,
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
    model_ident: &Ident,
) -> Vec<TokenStream> {
    let mut impls = Vec::new();
    let field_map: std::collections::HashMap<_, _> =
        fields.iter().map(|f| (f.ident.as_ref().unwrap(), f)).collect();

    for (_, (ref_cols, keys)) in groups {
        let idx_type = quote! {( #(#ref_cols,)* )};

        // Type of individual iterator items (flattened nested tuple of references)
        let item_type = quote! {
            <<<<#idx_type as ::diesel_builders::tuplities::NestTuple>::Nested
                as ::diesel_builders::TypedNestedTuple>::NestedTupleValueType
                as ::diesel_builders::tuplities::NestedTupleRef>::Ref<'a>
                as ::diesel_builders::tuplities::FlattenNestedTuple>::Flattened
        };

        // Build the chained iterator type and expression
        let (chain_iter_type, chain_expr) = build_chain_iterator(&keys, &field_map, &item_type);

        impls.push(quote! {
            impl ::diesel_builders::IterForeignKey<#idx_type> for #model_ident {
                type ForeignKeysIter<'a> = #chain_iter_type;

                fn iter_foreign_key(&self) -> Self::ForeignKeysIter<'_> {
                    #chain_expr
                }
            }
        });
    }

    impls
}

/// Builds a chained iterator type and expression for multiple foreign key
/// instances.
fn build_chain_iterator(
    keys: &[CapturedForeignKey],
    field_map: &std::collections::HashMap<&syn::Ident, &Field>,
    item_type: &TokenStream,
) -> (TokenStream, TokenStream) {
    let mut chain_iter_type = quote!(::std::iter::Empty<#item_type>);
    let mut chain_expr = quote!(::std::iter::empty());

    for key in keys {
        let (iter_expr, iter_type) = build_single_key_iterator(key, field_map, item_type);

        chain_iter_type = quote!(::std::iter::Chain<#chain_iter_type, #iter_type>);
        chain_expr = quote!(#chain_expr.chain(#iter_expr));
    }

    (chain_iter_type, chain_expr)
}

/// Builds an iterator expression and type for a single foreign key instance.
fn build_single_key_iterator(
    key: &CapturedForeignKey,
    field_map: &std::collections::HashMap<&syn::Ident, &Field>,
    item_type: &TokenStream,
) -> (TokenStream, TokenStream) {
    // Identify which fields are Options that need unwrapping
    let options_to_filter: Vec<_> = key
        .host_fields
        .iter()
        .enumerate()
        .filter(|(i, f_ident)| {
            let field = field_map[f_ident];
            key.unwrap_if_option[*i] && crate::utils::is_option(&field.ty)
        })
        .map(|(_, f_ident)| f_ident)
        .collect();

    let has_options = !options_to_filter.is_empty();

    if has_options {
        // Build a chain of .zip() calls to combine all Options
        let mut zip_expr = quote!(Some(()));
        let mut map_pattern = quote!(());

        for opt_ident in &options_to_filter {
            zip_expr = quote!(#zip_expr.zip(self.#opt_ident.as_ref()));
            map_pattern = quote!((#map_pattern, #opt_ident));
        }

        // Build the tuple of values, using unwrapped values where applicable
        let value_tokens: Vec<_> = key
            .host_fields
            .iter()
            .map(|f_ident| {
                if options_to_filter.contains(&f_ident) {
                    quote!(#f_ident) // Use unwrapped value from map pattern
                } else {
                    quote!(&self.#f_ident) // Use field directly
                }
            })
            .collect();

        let tuple_expr = quote!((#(#value_tokens,)*));

        let iter_expr = quote! {
            #zip_expr.map(|#map_pattern| #tuple_expr).into_iter()
        };
        let iter_type = quote!(::std::option::IntoIter<#item_type>);

        (iter_expr, iter_type)
    } else {
        // No Options to filter - yield a single item with all field references
        let value_tokens: Vec<_> =
            key.host_fields.iter().map(|f_ident| quote!(&self.#f_ident)).collect();

        let tuple_expr = quote!((#(#value_tokens,)*));
        let iter_expr = quote!(::std::iter::once(#tuple_expr));
        let iter_type = quote!(::std::iter::Once<#item_type>);

        (iter_expr, iter_type)
    }
}
