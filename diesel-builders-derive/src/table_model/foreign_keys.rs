//! Generate foreign key implementations for triangular relations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Ident};

use crate::table_model::attribute_parsing::{
    ForeignKeyAttribute, extract_discretionary_table, extract_mandatory_table,
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

    // Track host columns mapping to tables for FPK generation
    // Key: Host Column Ident (String)
    // Value: (Host Ident, List of unique Ref Table Paths)
    let mut host_col_to_refs: std::collections::HashMap<String, (syn::Ident, Vec<syn::Path>)> =
        std::collections::HashMap::new();

    // Pass 1: Collect candidates
    for fk in foreign_keys {
        let ref_cols = &fk.referenced_columns;

        if fk.host_columns.len() == 1 && ref_cols.len() == 1 {
            let host_col_ident = &fk.host_columns[0];
            if let Some(ref_table) = crate::utils::extract_table_path_from_column(&ref_cols[0]) {
                let entry = host_col_to_refs
                    .entry(host_col_ident.to_string())
                    .or_insert_with(|| (host_col_ident.clone(), Vec::new()));

                let ref_table_str = quote!(#ref_table).to_string();
                if !entry.1.iter().any(|t| quote!(#t).to_string() == ref_table_str) {
                    entry.1.push(ref_table);
                }
            }
        }
    }

    // Set of columns that will receive FPK implementation
    let fpk_column_names: std::collections::HashSet<String> = host_col_to_refs
        .iter()
        .filter(|(_, (_, tables))| tables.len() == 1)
        .map(|(k, _)| k.clone())
        .collect();

    // Pass 2: Generate implementations
    for fk in foreign_keys {
        let ref_cols = &fk.referenced_columns;

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

        // If this is a single column FK that will become an FPK, skip HostColumn
        // generation to avoid conflict with blanket implementation.
        if fk.host_columns.len() == 1 && fpk_column_names.contains(&fk.host_columns[0].to_string())
        {
            continue;
        }

        let host_cols_tokens: Vec<_> =
            fk.host_columns.iter().map(|c| quote!(#table_module::#c)).collect();

        for (idx, host_col_ident) in fk.host_columns.iter().enumerate() {
            let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
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

    // Pass 3: Generate FPKs for unique mappings
    for (_, (host_col_ident, tables)) in host_col_to_refs {
        if tables.len() == 1 {
            let ref_table = &tables[0];
            if crate::utils::should_generate_allow_tables_to_appear_in_same_query(
                &host_table_path,
                ref_table,
            ) {
                impls.push(
                    quote! { ::diesel::allow_tables_to_appear_in_same_query!(#table_module, #ref_table); },
                );
            }

            if let Some(stream) =
                generate_fpk_impl(&syn::parse_quote!(#table_module::#host_col_ident), ref_table)
            {
                impls.push(stream);
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
struct CapturedForeignKey<'a> {
    /// Host table field identifiers forming the foreign key
    host_fields: Vec<&'a Field>,
    /// Referenced column paths in the target table
    ref_cols: Vec<TokenStream>,
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
    let captured_keys = collect_foreign_keys(fields, foreign_keys)?;

    // Group foreign keys by their referenced index
    let groups = group_by_referenced_index(captured_keys.as_slice());

    // Generate an IterForeignKey impl for each unique referenced index
    Ok(generate_impls_for_groups(groups, table_module, model_ident))
}

/// Collects all foreign key relationships (both implicit and explicit).
fn collect_foreign_keys<'a>(
    fields: &'a syn::punctuated::Punctuated<Field, syn::token::Comma>,
    foreign_keys: &[ForeignKeyAttribute],
) -> syn::Result<Vec<CapturedForeignKey<'a>>> {
    let mut captured_keys = Vec::new();

    // Collect implicit foreign keys from triangular relations
    collect_triangular_foreign_keys(fields, &mut captured_keys)?;

    // Collect explicit foreign keys from attributes
    collect_explicit_foreign_keys(fields, foreign_keys, &mut captured_keys)?;

    Ok(captured_keys)
}

/// Collects foreign keys from mandatory/discretionary triangular relations.
fn collect_triangular_foreign_keys<'a>(
    fields: &'a syn::punctuated::Punctuated<Field, syn::token::Comma>,
    captured_keys: &mut Vec<CapturedForeignKey<'a>>,
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
                            host_fields: vec![field, other_field],
                            ref_cols: vec![ref_pk, quote!(#ref_col)],
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
fn collect_explicit_foreign_keys<'a>(
    fields: &'a syn::punctuated::Punctuated<Field, syn::token::Comma>,
    foreign_keys: &[ForeignKeyAttribute],
    captured_keys: &mut Vec<CapturedForeignKey<'a>>,
) -> syn::Result<()> {
    for fk in foreign_keys {
        let ref_cols_paths = &fk.referenced_columns;

        // Explicit multi-column foreign key
        if fk.host_columns.len() != ref_cols_paths.len() {
            return Err(syn::Error::new_spanned(
                &ref_cols_paths[0],
                "Mismatched number of host and referenced columns in foreign_key definition.",
            ));
        }

        let ref_cols_tokens: Vec<_> = ref_cols_paths.iter().map(|p| quote!(#p)).collect();

        // Create a unique grouping key from the referenced columns
        // This logic ensures that FKs targeting the same sets of columns are grouped
        // together for iteration.
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

        let mut host_fields = Vec::new();
        for host_col_ident in &fk.host_columns {
            let host_field = fields
                .iter()
                .find(|f| f.ident.as_ref() == Some(host_col_ident))
                .ok_or_else(|| {
                    syn::Error::new_spanned(
                        host_col_ident,
                        "Host field not found in struct definition.",
                    )
                })?;
            host_fields.push(host_field);
        }

        captured_keys.push(CapturedForeignKey {
            host_fields,
            ref_cols: ref_cols_tokens,
            grouping_key,
        });
    }
    Ok(())
}

/// Groups foreign keys by their referenced index.
fn group_by_referenced_index<'a, 'b>(
    captured_keys: &'b [CapturedForeignKey<'a>],
) -> std::collections::HashMap<&'b str, (&'b [TokenStream], Vec<&'b CapturedForeignKey<'a>>)> {
    use std::collections::HashMap;

    let mut groups: HashMap<&'b str, (&'b [TokenStream], Vec<&'b CapturedForeignKey<'a>>)> =
        HashMap::new();

    for key in captured_keys {
        groups
            .entry(key.grouping_key.as_str())
            .or_insert_with(|| (key.ref_cols.as_slice(), Vec::new()))
            .1
            .push(key);
    }

    groups
}

/// Generates `IterForeignKey` trait implementations for each group of foreign
/// keys.
fn generate_impls_for_groups<'b>(
    groups: std::collections::HashMap<
        &'b str,
        (&'b [TokenStream], Vec<&'b CapturedForeignKey<'_>>),
    >,
    table_module: &Ident,
    model_ident: &Ident,
) -> Vec<TokenStream> {
    let mut impls = Vec::new();

    for (_, (ref_cols, keys)) in groups {
        let idx_type = quote! {( #(#ref_cols,)* )};

        assert!(!keys.is_empty(), "Cannot generate iterator for empty key group");
        let first_key = keys[0];

        // Determine base types (inner types T for T or Option<T>) of the host fields
        // We use the first key as a template. All keys in group target same index,
        // implies they have compatible types.
        let mut base_types = Vec::new();
        for field in &first_key.host_fields {
            let ty = &field.ty;
            let inner_ty = if crate::utils::is_option(ty) {
                if let syn::Type::Path(type_path) = ty
                    && let Some(segment) = type_path.path.segments.last()
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
                {
                    inner
                } else {
                    ty // Fallback, shouldn't happen if is_option checks out
                }
            } else {
                ty
            };
            base_types.push(quote!(#inner_ty));
        }

        // Build item types for the iterators (Nested Tuples)

        // MatchSimpleIter: Nested tuple of Option<&T>
        let simple_item_types: Vec<_> =
            base_types.iter().map(|ty| quote!(::std::option::Option<&'a #ty>)).collect();
        let simple_elem_ty = recursive_tuple_type(&simple_item_types);

        // MatchFullIter: Nested tuple of &T
        let full_item_types: Vec<_> = base_types.iter().map(|ty| quote!(&'a #ty)).collect();
        let full_elem_ty = recursive_tuple_type(&full_item_types);

        // Build chains
        // (type_simple, expr_simple, type_full, expr_full)
        let (simple_iter_type, simple_iter_expr, full_iter_type, full_iter_expr) =
            build_chain_iterators(&keys, &simple_elem_ty, &full_elem_ty, table_module);

        // Build ForeignKeyItemType: Nested tuple of Box<dyn ...>
        let boxed_column_types: Vec<_> = keys[0]
            .host_fields
            .iter()
            .map(|f| {
                let f_ident = f.ident.as_ref().unwrap();
                let host_col = quote!(#table_module::#f_ident);
                quote!(::std::boxed::Box<dyn ::diesel_builders::DynTypedColumn<
                ValueType = <#host_col as ::diesel_builders::ValueTyped>::ValueType,
                Table = #table_module::table,
            >>)
            })
            .collect();
        let foreign_key_item_type = recursive_tuple_type(&boxed_column_types);

        let foreign_keys_expr = build_foreign_keys_iterator(keys.as_slice(), table_module);

        impls.push(quote! {
            impl ::diesel_builders::IterForeignKey<#idx_type> for #model_ident {
                type MatchSimpleIter<'a> = #simple_iter_type
                where
                    #idx_type: 'a,
                    Self: 'a;

                type MatchFullIter<'a> = #full_iter_type
                where
                    #idx_type: 'a,
                    Self: 'a;

                type ForeignKeyItemType = #foreign_key_item_type;

                type ForeignKeyColumnsIter = ::std::vec::IntoIter<Self::ForeignKeyItemType>;

                fn iter_match_simple<'a>(&'a self) -> Self::MatchSimpleIter<'a>
                    where #idx_type: 'a
                {
                    #simple_iter_expr
                }

                fn iter_match_full<'a>(&'a self) -> Self::MatchFullIter<'a>
                    where #idx_type: 'a
                {
                    #full_iter_expr
                }

                fn iter_foreign_key_columns() -> Self::ForeignKeyColumnsIter {
                    #foreign_keys_expr
                }
            }
        });
    }

    impls
}

/// Builds chained iterator types and expressions for multiple foreign key
/// instances. Returns (`SimpleType`, `SimpleExpr`, `FullType`, `FullExpr`).
fn build_chain_iterators(
    keys: &[&CapturedForeignKey<'_>],
    simple_elem_ty: &TokenStream,
    full_elem_ty: &TokenStream,
    table_module: &Ident,
) -> (TokenStream, TokenStream, TokenStream, TokenStream) {
    let mut simple_iter_type = quote!(::std::iter::Empty<#simple_elem_ty>);
    let mut simple_iter_expr = quote!(::std::iter::empty());

    let mut full_iter_type = quote!(::std::iter::Empty<#full_elem_ty>);
    let mut full_iter_expr = quote!(::std::iter::empty());

    for key in keys {
        let (s_expr, s_type, f_expr, f_type) =
            build_single_key_iterators(key, simple_elem_ty, full_elem_ty, table_module);

        simple_iter_type = quote!(::std::iter::Chain<#simple_iter_type, #s_type>);
        simple_iter_expr = quote!(#simple_iter_expr.chain(#s_expr));

        full_iter_type = quote!(::std::iter::Chain<#full_iter_type, #f_type>);
        full_iter_expr = quote!(#full_iter_expr.chain(#f_expr));
    }

    (simple_iter_type, simple_iter_expr, full_iter_type, full_iter_expr)
}

/// Builds iterator expressions and types for a single foreign key instance.
fn build_single_key_iterators(
    key: &CapturedForeignKey<'_>,
    simple_elem_ty: &TokenStream,
    full_elem_ty: &TokenStream,
    table_module: &Ident,
) -> (TokenStream, TokenStream, TokenStream, TokenStream) {
    // Simple Iterator: Always yields `Option<&T>` (nested tuple).
    let mut simple_val_tokens = Vec::new();

    // Full Iterator: Yields `&T` or skips if any column is missing.
    // We construct a match guard: (val1, val2) -> Some((v1, v2)) or None
    let mut full_match_arms = Vec::new();
    let mut full_construction_vars = Vec::new();

    for (i, field) in key.host_fields.iter().enumerate() {
        let field_ident = field.ident.as_ref().unwrap();
        let col_path = quote!(#table_module::#field_ident);
        let accessor = quote!(::diesel_builders::GetColumn::<#col_path>::get_column_ref(self));
        // Note: accessors borrow self.

        let is_optional = crate::utils::is_option(&field.ty);

        let var_name = syn::Ident::new(&format!("v_{i}"), proc_macro2::Span::call_site());

        if is_optional {
            simple_val_tokens.push(quote!(#accessor.as_ref()));
            full_match_arms
                .push((quote!(#accessor), quote!(::std::option::Option::Some(#var_name))));
        } else {
            simple_val_tokens.push(quote!(::std::option::Option::Some(#accessor)));
            full_match_arms.push((quote!(#accessor), quote!(#var_name)));
        }
        full_construction_vars.push(quote!(#var_name));
    }

    // Simple Iter
    let simple_tuple_expr = recursive_tuple_expr(&simple_val_tokens);
    let simple_iter_expr = quote!(::std::iter::once(#simple_tuple_expr));
    let simple_iter_type = quote!(::std::iter::Once<#simple_elem_ty>);

    // Full Iter: match (...) { (Some(v), ...) => Some(nested_tuple), _ => None }
    let match_exprs: Vec<_> = full_match_arms.iter().map(|(e, _)| e).collect();
    let match_pats: Vec<_> = full_match_arms.iter().map(|(_, p)| p).collect();

    // Tuple of expressions: (&self.f1, self.f2_ref, ...)
    let match_target = quote!((#(#match_exprs,)*));
    // Tuple pattern: (Some(v0), v1, ...)
    let match_pattern = quote!((#(#match_pats,)*));

    let full_tuple_val = recursive_tuple_expr(&full_construction_vars);

    let full_opt_expr = quote! {
        match #match_target {
            #match_pattern => ::std::option::Option::Some(#full_tuple_val),
            _ => ::std::option::Option::None,
        }
    };

    let full_iter_expr = quote!(::std::option::Option::into_iter(#full_opt_expr));
    let full_iter_type = quote!(::std::option::IntoIter<#full_elem_ty>);

    (simple_iter_expr, simple_iter_type, full_iter_expr, full_iter_type)
}

/// Builds an iterator expression that returns column tuples (Nested Tuples of
/// Boxes).
fn build_foreign_keys_iterator(
    keys: &[&CapturedForeignKey<'_>],
    table_module: &syn::Ident,
) -> TokenStream {
    let mut items = Vec::new();

    for key in keys {
        // For each foreign key, create a tuple of HOST table column instances
        let host_columns: Vec<_> = key
            .host_fields
            .iter()
            .map(|host_field| {
                let name = host_field.ident.as_ref().unwrap();
                let host_col = quote!(#table_module::#name);
                quote! {
                    ::std::boxed::Box::new(#host_col) as ::std::boxed::Box<dyn ::diesel_builders::DynTypedColumn<
                    ValueType = <#host_col as ::diesel_builders::ValueTyped>::ValueType,
                    Table = #table_module::table,
                >>
                }
            })
            .collect();

        items.push(recursive_tuple_expr(&host_columns));
    }

    quote! {
        ::std::vec![#(#items),*].into_iter()
    }
}

// Helpers for nested tuples
/// Recursively builds a nested tuple type from a slice of types.
/// `[A, B, C]` -> `(A, (B, (C,)))` (with unit termination if needed, or
/// specific structure) Actually implementation logic:
/// `[]` -> `()`
/// `[single]` -> `(single,)`
/// `[head, tail...]` -> `(head, tail_recursion)`
/// e.g. `[A, B, C]` -> `(A, (B, (C,)))`
fn recursive_tuple_type(types: &[TokenStream]) -> TokenStream {
    match types {
        [] => quote!(()),
        [single] => quote!((#single,)),
        [head, tail @ ..] => {
            let tail_tokens = recursive_tuple_type(tail);
            quote!((#head, #tail_tokens))
        }
    }
}

/// Recursively builds a nested tuple expression from a slice of expressions.
fn recursive_tuple_expr(exprs: &[TokenStream]) -> TokenStream {
    match exprs {
        [] => quote!(()),
        [single] => quote!((#single,)),
        [head, tail @ ..] => {
            let tail_tokens = recursive_tuple_expr(tail);
            quote!((#head, #tail_tokens))
        }
    }
}
