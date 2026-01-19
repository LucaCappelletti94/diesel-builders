//! Module for `TableModel` derive macro implementation.
//!
//! This module contains the implementation of the `TableModel` derive macro,
//! split into logical components for better maintainability.

mod attribute_parsing;
mod foreign_keys;
mod get_column;
mod may_get_columns;
mod primary_key;
mod set_columns;
mod table_generation;
mod typed_column;
mod vertical_same_as;

use std::collections::HashMap;

use attribute_parsing::{
    extract_discretionary_table, extract_field_default_value, extract_mandatory_table,
    extract_primary_key_columns, extract_same_as_columns, extract_table_model_attributes,
    extract_table_module, is_field_discretionary, is_field_infallible, is_field_mandatory,
    validate_field_attributes,
};
use foreign_keys::{
    generate_explicit_foreign_key_impls, generate_foreign_key_impls,
    generate_iter_foreign_key_impls,
};
use get_column::generate_get_column_impls;
use primary_key::generate_indexed_column_impls;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, spanned::Spanned};
use table_generation::generate_table_macro;
use typed_column::generate_typed_column_impls;
use vertical_same_as::generate_vertical_same_as_impls;

use crate::utils::{format_as_nested_tuple, is_option};

/// Helper to convert `TokenStream` to normalized string for comparison.
fn tokens_to_string(tokens: &impl quote::ToTokens) -> String {
    quote::quote!(#tokens).to_string().replace(' ', "")
}

/// Struct to hold processed field information.
struct ProcessedFields {
    /// Columns for the new record tuple.
    new_record_columns: Vec<syn::Path>,
    /// Records that are infallible (index, path).
    infallible_records: Vec<syn::Path>,
    /// Default values for fields.
    default_values: Vec<proc_macro2::TokenStream>,
    /// Warnings to be emitted.
    warnings: Vec<proc_macro2::TokenStream>,
}

/// Process fields to extract columns, validation status, and default values.
#[allow(clippy::too_many_lines)]
fn process_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
    primary_key_columns: &[syn::Ident],
    attributes: &attribute_parsing::TableModelAttributes,
) -> syn::Result<ProcessedFields> {
    let mut new_record_columns = Vec::new();
    let mut infallible_records = Vec::new();
    let mut default_values = Vec::new();
    let mut warnings = Vec::new();

    for field in fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;

        // Check if field is a primary key
        let is_pk = primary_key_columns.iter().any(|pk| pk == field_name);

        if is_pk {
            if extract_field_default_value(field).is_some() && attributes.surrogate_key {
                return Err(syn::Error::new_spanned(
                    field,
                    "Surrogate primary key cannot have a `default` value",
                ));
            }

            if is_field_infallible(field) && attributes.surrogate_key {
                return Err(syn::Error::new_spanned(
                    field,
                    "Surrogate primary key cannot be marked as `#[infallible]`",
                ));
            }
        }

        if is_pk && attributes.surrogate_key {
            continue;
        }

        new_record_columns.push(syn::parse_quote!(#table_module::#field_name));

        if is_field_infallible(field) && attributes.error.is_none() {
            let warning_msg = format!(
                "Field `{field_name}` is marked `#[infallible]` but the `TableModel` does not specify an error type, making the attribute redundant.",
            );

            let mut span = field.span();
            for attr in &field.attrs {
                if attr.path().is_ident("infallible") {
                    span = attr.span();
                    break;
                }
                if attr.path().is_ident("table_model") {
                    let mut found = false;
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("infallible") {
                            found = true;
                        }
                        Ok(())
                    });
                    if found {
                        span = attr.span();
                        break;
                    }
                }
            }

            let const_name =
                syn::Ident::new(&format!("__WARN_REDUNDANT_INFALLIBLE_{field_name}"), span);
            warnings.push(quote! {
                const _: () = {
                    #[deprecated(note = #warning_msg)]
                    #[allow(non_upper_case_globals)]
                    const #const_name: () = ();
                    let _ = #const_name;
                };
            });
        }

        if is_field_infallible(field) || attributes.error.is_none() {
            infallible_records.push(syn::parse_quote!(#table_module::#field_name));
        }

        // Default value logic
        let user_default = extract_field_default_value(field);
        let is_nullable = is_option(&field.ty);

        let default_val = if let Some(def) = user_default {
            quote::quote! { Some((#def).to_owned().into()) }
        } else if is_nullable {
            quote::quote! { Some(None) }
        } else {
            quote::quote! { None }
        };
        default_values.push(default_val);
    }

    Ok(ProcessedFields { new_record_columns, infallible_records, default_values, warnings })
}

/// Collect mandatory and discretionary triangular relation columns.
fn collect_triangular_columns(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
) -> (Vec<syn::Type>, Vec<syn::Type>) {
    let mut mandatory_columns = Vec::new();
    let mut discretionary_columns = Vec::new();
    fields.iter().for_each(|field| {
        let Some(field_name) = field.ident.as_ref() else {
            return;
        };
        let col = syn::parse_quote!(#table_module::#field_name);

        if is_field_mandatory(field) {
            mandatory_columns.push(col);
        } else if is_field_discretionary(field) {
            discretionary_columns.push(col);
        }
    });

    (mandatory_columns, discretionary_columns)
}

/// Collect tables referenced by mandatory and discretionary fields.
/// Returns a set of unique table paths.
fn collect_triangular_relation_tables(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> syn::Result<HashMap<&syn::Ident, syn::Path>> {
    use attribute_parsing::{extract_discretionary_table, extract_mandatory_table};

    let mut referenced_tables = HashMap::with_capacity(fields.len());

    for field in fields {
        if let Some(field_name) = &field.ident {
            // Check if field is mandatory and extract its referenced table
            if is_field_mandatory(field) {
                if let Some(table_path) = extract_mandatory_table(field)? {
                    referenced_tables.insert(field_name, table_path);
                } else {
                    return Err(syn::Error::new_spanned(
                        field,
                        format!(
                            "Field '{field_name}' is marked as #[mandatory] but no table name is specified. Use #[mandatory(table_name)]",
                        ),
                    ));
                }
            }

            // Check if field is discretionary and extract its referenced table
            if is_field_discretionary(field) {
                if let Some(table_path) = extract_discretionary_table(field)? {
                    referenced_tables.insert(field_name, table_path);
                } else {
                    return Err(syn::Error::new_spanned(
                        field,
                        format!(
                            "Field '{field_name}' is marked as #[discretionary] but no table name is specified. Use #[discretionary(table_name)]"
                        ),
                    ));
                }
            }
        }
    }

    Ok(referenced_tables)
}

/// Collect tables referenced by mandatory and discretionary fields.
/// Returns a set of unique table paths.
fn collect_unique_triangular_relation_tables(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> syn::Result<Vec<syn::Path>> {
    let tables = collect_triangular_relation_tables(fields)?;
    let mut observed_table_idents = Vec::new();
    let mut observed_tables = Vec::new();
    for table in tables.values() {
        if let Some(last_segment) = table.segments.last()
            && !observed_table_idents.contains(&last_segment)
        {
            observed_table_idents.push(last_segment);
            observed_tables.push(table.clone());
        }
    }
    Ok(observed_tables)
}

/// Generate fpk! implementations for mandatory and discretionary fields.
/// Returns a vector of `TokenStream`s, one for each field.
fn generate_triangular_fpk_impls(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut fpk_impls = Vec::new();

    for (field_name, triangular_table) in collect_triangular_relation_tables(fields)? {
        // Generate fpk implementation using the fpk generation function
        let column_path: syn::Path = syn::parse_quote!(#table_module::#field_name);
        fpk_impls.extend(foreign_keys::generate_fpk_impl(&column_path, &triangular_table));
    }

    Ok(fpk_impls)
}

/// Information about a horizontal key.
struct HorizontalKeyInfo {
    /// The field representing the key.
    field: syn::Ident,
    /// The path to the key column.
    key_column: syn::Path,
    /// Whether the key is mandatory.
    is_mandatory: bool,
    /// The columns in the host table that are part of the key.
    host_columns: Vec<syn::Ident>,
    /// The columns in the foreign table that are part of the key.
    foreign_columns: Vec<syn::Path>,
}

/// Main entry point for the `TableModel` derive macro.
#[allow(clippy::too_many_lines)]
pub fn derive_table_model_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;

    // Parse attributes
    let table_module_opt = extract_table_module(input);
    let primary_key_columns = extract_primary_key_columns(input);
    let attributes = extract_table_model_attributes(input)?;

    let table_module = if let Some(module) = table_module_opt {
        module
    } else {
        let struct_name = struct_ident.to_string();
        let table_name_str = format!("{}s", crate::utils::camel_to_snake_case(&struct_name));
        syn::Ident::new(&table_name_str, struct_ident.span())
    };

    if let Some(ancestors) = &attributes.ancestors {
        let table_type_str = table_module.to_string();

        let mut seen = std::collections::HashSet::with_capacity(ancestors.len());
        for ancestor in ancestors {
            let ancestor_str = tokens_to_string(ancestor);

            if ancestor_str == table_type_str {
                return Err(syn::Error::new_spanned(
                    ancestor,
                    "Table cannot be its own `ancestor`",
                ));
            }

            if !seen.insert(ancestor_str) {
                return Err(syn::Error::new_spanned(ancestor, "Duplicate `ancestor` in hierarchy"));
            }
        }
    }

    if attributes.surrogate_key && primary_key_columns.len() > 1 {
        return Err(syn::Error::new_spanned(
            input,
            "`surrogate_key` is not supported for composite primary keys",
        ));
    }

    // Extract fields
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

    // Validate that all primary key columns exist in the struct
    let field_names: Vec<_> = fields.iter().filter_map(|f| f.ident.as_ref()).collect();

    for pk_column in &primary_key_columns {
        if !field_names.contains(&pk_column) {
            return Err(syn::Error::new_spanned(
                input,
                format!(
                    "Primary key column `{pk_column}` not found in struct. \
                     `TableModel` requires a detectable primary key. Either:\n\
                     1. Add an `id` field to your struct (default primary key), or\n\
                     2. Specify primary key columns with `#[diesel(primary_key(your_column))]`",
                ),
            ));
        }
    }

    // Validate fields before generation to ensure unsupported attributes are
    // reported correctly
    for field in fields {
        validate_field_attributes(field)?;
    }

    // Generate all components
    let table_macro = generate_table_macro(input, &table_module, &primary_key_columns)?;
    let typed_column_impls =
        generate_typed_column_impls(fields, &table_module, struct_ident, &primary_key_columns);
    let get_column_impls = generate_get_column_impls(fields, &table_module, struct_ident);
    let indexed_column_impls = generate_indexed_column_impls(&table_module, &primary_key_columns);
    let nested_primary_keys = format_as_nested_tuple(
        primary_key_columns.iter().map(|col| quote::quote! { #table_module::#col }),
    );

    let ProcessedFields { new_record_columns, infallible_records, default_values, warnings } =
        process_fields(fields, &table_module, &primary_key_columns, &attributes)?;

    // Collect triangular relation columns for BundlableTable implementation
    let (mandatory_columns, discretionary_columns) =
        collect_triangular_columns(fields, &table_module);

    // Validate that surrogate keys don't have triangular relations
    if attributes.surrogate_key
        && (!mandatory_columns.is_empty() || !discretionary_columns.is_empty())
    {
        return Err(syn::Error::new_spanned(
            input,
            "Tables with `surrogate_key` cannot have `#[mandatory]` or `#[discretionary]` attributes. \
             Surrogate keys are auto-generated and cannot participate in triangular relations.",
        ));
    }

    // Validate mandatory triangular relations on primary keys
    for field in fields {
        if is_field_mandatory(field)
            && let Some(mandatory_table) = extract_mandatory_table(field)?
        {
            // Check if ALL primary key columns have a same_as pointing to this mandatory
            // table
            for pk_col_name in &primary_key_columns {
                let pk_field = fields.iter().find(|f| f.ident.as_ref() == Some(pk_col_name));

                if let Some(pk_field) = pk_field {
                    let same_as_cols_groups = extract_same_as_columns(pk_field)?;

                    let has_same_as_to_mandatory =
                        same_as_cols_groups.iter().flatten().any(|path| {
                            let number_of_segments = path.segments.len();
                            assert!(
                                number_of_segments >= 2,
                                "Column path in #[same_as(...)] must be in the format `table::column`"
                            );
                            let col_table = &path.segments[number_of_segments - 2];
                            let mandatory_table = &mandatory_table.segments.last().unwrap();
                            col_table.ident == mandatory_table.ident
                        });

                    if !has_same_as_to_mandatory {
                        let mandatory_table_str = tokens_to_string(&mandatory_table);
                        return Err(syn::Error::new_spanned(
                            pk_field,
                            format!(
                                "Primary key column `{pk_col_name}` must have a `#[same_as({mandatory_table_str}::Column)]` attribute \
                                     specifying the corresponding column in the mandatory table `{mandatory_table_str}`.",
                            ),
                        ));
                    }
                }
            }
        }
    }

    // Collect tables referenced by triangular relations
    let triangular_relation_tables = collect_unique_triangular_relation_tables(fields)?;

    // Generate `fpk!` implementations for triangular relation fields
    let triangular_fpk_impls = generate_triangular_fpk_impls(fields, &table_module)?;

    // Generate `allow_tables_to_appear_in_same_query!` macro calls for ancestors
    // and triangular relations
    let table_name = table_module.to_string();
    let table_module_path: syn::Path = table_module.clone().into();
    let allow_same_query_calls = attributes
        .ancestors
        .iter()
        .flat_map(|paths| paths.iter())
        .chain(triangular_relation_tables.iter())
        .filter_map(|other| {
            if crate::utils::should_generate_allow_tables_to_appear_in_same_query(
                &table_module_path,
                other,
            ) {
                Some(quote! {
                    ::diesel::allow_tables_to_appear_in_same_query!(#table_module, #other);
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let new_record = format_as_nested_tuple(&new_record_columns);
    let default_new_record = format_as_nested_tuple(&default_values);
    let new_record_type =
        format_as_nested_tuple(new_record_columns.iter().map(
            |col| quote::quote! { Option<<#col as ::diesel_builders::ColumnTyped>::ColumnType> },
        ));
    let may_get_column_impls =
        may_get_columns::generate_may_get_column_impls(&new_record_columns, &table_module);

    let infallible_validate_column_impls =
        set_columns::generate_infallible_validate_column_impls(&infallible_records, &table_module);

    let set_column_impls =
        set_columns::generate_set_column_impls(&new_record_columns, &table_module);

    let error_type = attributes
        .error
        .as_ref()
        .map(|t| quote::quote! { #t })
        .unwrap_or(quote::quote! { std::convert::Infallible });

    // Generate Root/Descendant implementations
    // If ancestors are specified, generate Descendant; otherwise generate Root
    let descendant_impls = if let Some(ref ancestors) = attributes.ancestors {
        let table_type: syn::Type = syn::parse_quote!(#table_module::table);
        // Convert ancestor module paths to table types for the trait implementation
        let ancestor_tables: Vec<syn::Type> =
            ancestors.iter().map(|a| syn::parse_quote!(#a::table)).collect();
        let root: &syn::Type = ancestor_tables.first().unwrap();
        let aux_impls =
            crate::descendant::generate_auxiliary_descendant_impls(&table_type, &ancestor_tables);

        quote! {
            impl ::diesel_builders::Descendant for #table_type {
                type Ancestors = (#(#ancestor_tables,)*);
                type Root = #root;
            }
            #aux_impls
        }
    } else {
        // No ancestors attribute means this is a root table
        let table_type: syn::Type = syn::parse_quote!(#table_module::table);
        let aux_impls = crate::descendant::generate_auxiliary_descendant_impls(&table_type, &[]);

        quote! {
            impl ::diesel_builders::Root for #table_type {}

            impl ::diesel_builders::Descendant for #table_type {
                type Ancestors = ();
                type Root = Self;
            }

            #aux_impls
        }
    };

    let bundlable_table_impl = quote! {
        impl ::diesel_builders::BundlableTable for #table_module::table {
            type MandatoryTriangularColumns = (#(#mandatory_columns,)*);
            type DiscretionaryTriangularColumns = (#(#discretionary_columns,)*);
        }
    };

    // Generate MandatorySameAsIndex implementations for mandatory columns
    let mandatory_same_as_impls: Vec<_> = mandatory_columns
        .iter()
        .enumerate()
        .map(|(i, column)| {
            let idx = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            quote! {
                impl ::diesel_builders::MandatorySameAsIndex for #column {
                    type Idx = ::diesel_builders::typenum::#idx;
                }
            }
        })
        .collect();

    // Generate DiscretionarySameAsIndex implementations for discretionary columns
    let discretionary_same_as_impls: Vec<_> = discretionary_columns
        .iter()
        .enumerate()
        .map(|(i, column)| {
            let idx = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            quote! {
                impl ::diesel_builders::DiscretionarySameAsIndex for #column {
                    type Idx = ::diesel_builders::typenum::#idx;
                }
            }
        })
        .collect();

    // Collect Horizontal Keys
    // Map from TargetTable (last segment ident) to list of (KeyField, IsMandatory,
    // TargetTablePath)
    let mut potential_keys: HashMap<syn::Ident, Vec<(&syn::Ident, bool, syn::Path)>> =
        HashMap::new();

    for field in fields {
        let Some(field_name) = &field.ident else {
            continue;
        };

        let (target_table, is_mandatory) = if is_field_mandatory(field) {
            (extract_mandatory_table(field)?, true)
        } else if is_field_discretionary(field) {
            (extract_discretionary_table(field)?, false)
        } else {
            continue;
        };

        if let Some(target_table) = target_table
            && let Some(last_segment) = target_table.segments.last()
        {
            potential_keys.entry(last_segment.ident.clone()).or_default().push((
                field_name,
                is_mandatory,
                target_table,
            ));
        }
    }

    // Initialize horizontal_keys map: KeyField -> HorizontalKeyInfo
    let mut horizontal_keys_map: HashMap<&syn::Ident, HorizontalKeyInfo> = HashMap::new();

    for keys in potential_keys.values() {
        for (key_field, is_mandatory, _) in keys {
            horizontal_keys_map.insert(
                key_field,
                HorizontalKeyInfo {
                    field: (*key_field).clone(),
                    key_column: syn::parse_quote!(#table_module::#key_field),
                    is_mandatory: *is_mandatory,
                    host_columns: Vec::new(),
                    foreign_columns: Vec::new(),
                },
            );
        }
    }

    for f in fields {
        if let Ok(same_as_attributes) = extract_same_as_columns(f) {
            for attr_paths in same_as_attributes {
                // Check for explicit key in the attribute (2nd argument)
                let explicit_key_ident = if let [_, potential_key_path] = &attr_paths[..]
                    && let Some(segment) = potential_key_path.segments.last()
                    && horizontal_keys_map.contains_key(&segment.ident)
                {
                    Some(segment.ident.clone())
                } else {
                    None
                };

                for (i, col_path) in attr_paths.iter().enumerate() {
                    // If this is the explicit key, skip it (it's not a target column)
                    if let Some(k) = &explicit_key_ident
                        && i == 1
                        && col_path.segments.last().map(|s| &s.ident) == Some(k)
                    {
                        continue;
                    }

                    let number_of_segments = col_path.segments.len();
                    if number_of_segments < 2 {
                        return Err(syn::Error::new_spanned(
                            col_path,
                            "Non-key column path in #[same_as(...)] must be in the format `table::column` or a `#[mandatory]`/`#[discretionary]` attribute is missing.",
                        ));
                    }
                    let table_ident = &col_path.segments[number_of_segments - 2].ident;

                    // Check if this matches a target table
                    if let Some(keys) = potential_keys.get(table_ident) {
                        // Found a match for target table

                        let selected_key: Option<Ident> = if let Some(k_ident) = &explicit_key_ident
                        {
                            // Verify the explicit key belongs to this target table
                            if keys.iter().any(|(kf, _, _)| kf == &k_ident) {
                                Some(k_ident.clone())
                            } else {
                                // Explicit key provided but doesn't match this target table
                                // This might happen if we have #[same_as(Target1, KeyForTarget2)]
                                // We ignore it for Target1.
                                None
                            }
                        } else if keys.len() == 1 {
                            Some(keys[0].0.clone())
                        } else {
                            // Ambiguous
                            let available_keys: Vec<String> =
                                keys.iter().map(|(k, _, _)| format!("`{k}`")).collect();
                            let available_keys_str = available_keys.join(", ");
                            let col_name = &col_path.segments.last().unwrap().ident;

                            return Err(syn::Error::new_spanned(
                                f,
                                format!(
                                    "Ambiguous triangular relationship: multiple fields point to table `{table_ident}`. \
                                            Please specify which key to use: `#[same_as({table_ident}::{col_name}, KeyField)]`. \
                                            Available keys: {available_keys_str}"
                                ),
                            ));
                        };

                        if let Some(key_ident) = selected_key
                            && let Some(info) = horizontal_keys_map.get_mut(&key_ident)
                            && let Some(f_ident) = &f.ident
                        {
                            info.host_columns.push(f_ident.clone());
                            info.foreign_columns.push(col_path.clone());
                        }
                    }
                }
            }
        }
    }

    let horizontal_keys: Vec<_> = horizontal_keys_map.into_values().collect();
    // We do not filter out keys with no columns, as they still need to implement
    // HorizontalKey to satisfy BundlableTable bounds, even if they don't
    // propagate any values.

    // Generate HorizontalKey implementations
    let mut horizontal_key_impls = Vec::with_capacity(horizontal_keys.len());
    for key in &horizontal_keys {
        let mut seen = std::collections::HashSet::with_capacity(key.foreign_columns.len());
        for foreign_col in &key.foreign_columns {
            let col_str = tokens_to_string(foreign_col);
            if !seen.insert(col_str.clone()) {
                let err = syn::Error::new_spanned(
                    foreign_col,
                    format!(
                        "Duplicate column in ForeignColumns: `{col_str}`. \
                         This column appears multiple times in the same horizontal key relationship. \
                         Please ensure that each column is only involved in one `same_as` relationship for this key."
                    ),
                );
                horizontal_key_impls.push(err.to_compile_error());
            }
        }

        let key_column = &key.key_column;

        // We check that the key has at least one host and one foreign column, or
        // raise an appropriate compile-time error.
        if key.host_columns.is_empty() {
            return Err(syn::Error::new_spanned(
                key.field.clone(),
                "Horizontal key must have at least one host column. \
                 No host columns were found for this key. \
                 Please ensure that at least one field in this struct has a `#[same_as(...)]` attribute referencing this key.",
            ));
        }

        if key.foreign_columns.is_empty() {
            return Err(syn::Error::new_spanned(
                key.field.clone(),
                "Horizontal key must have at least one foreign column. \
                 No foreign columns were found for this key. \
                 Please ensure that at least one field in this struct has a `#[same_as(...)]` attribute referencing this key.",
            ));
        }

        let host_cols: Vec<_> =
            key.host_columns.iter().map(|f| quote::quote!(#table_module::#f)).collect();
        let foreign_cols = &key.foreign_columns;

        horizontal_key_impls.push(quote! {
            impl ::diesel_builders::HorizontalKey for #key_column {
                type HostColumns = (#(#host_cols,)*);
                type ForeignColumns = (#(#foreign_cols,)*);
            }
        });
    }

    // Generate HorizontalSameAsGroup for each column
    let column_horizontal_impls: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;

            // Find keys where this field is a host column
            let mut mandatory_keys = Vec::new();
            let mut discretionary_keys = Vec::new();
            let mut idx: Option<usize> = None;

            for key in &horizontal_keys {
                if let Some(pos) = key.host_columns.iter().position(|f| f == field_name) {
                    if let Some(existing_idx) = idx {
                        if existing_idx != pos {
                            // Index mismatch - this is a limitation of
                            // HorizontalSameAsGroup
                            // For now, we can't support this case easily
                            // without more complex logic
                            // But usually fields are in consistent order.
                            // We'll just use the first one found and hope for
                            // the best or error?
                            // Let's assume consistency for now.
                        }
                    } else {
                        idx = Some(pos);
                    }

                    if key.is_mandatory {
                        mandatory_keys.push(&key.key_column);
                    } else {
                        discretionary_keys.push(&key.key_column);
                    }
                }
            }

            let idx_type = if let Some(i) = idx {
                let idx_ident = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
                quote! { ::diesel_builders::typenum::#idx_ident }
            } else {
                quote! { ::diesel_builders::typenum::U0 }
            };

            Some(quote! {
                impl ::diesel_builders::HorizontalSameAsGroup for #table_module::#field_name {
                    type Idx = #idx_type;
                    type MandatoryHorizontalKeys = (#(#mandatory_keys,)*);
                    type DiscretionaryHorizontalKeys = (#(#discretionary_keys,)*);
                }
            })
        })
        .collect();

    // Generate VerticalSameAsGroup implementations for all columns
    let vertical_same_as_impls = generate_vertical_same_as_impls(
        fields,
        &table_module,
        &attributes,
        &triangular_relation_tables,
    )?;

    // Generate foreign key implementations for triangular relations
    let foreign_key_impls = generate_foreign_key_impls(fields, &table_module)?;

    // Generate explicit foreign key implementations
    let explicit_foreign_key_impls =
        generate_explicit_foreign_key_impls(&attributes.foreign_keys, &table_module)?;

    // Generate IterForeignKey implementations
    let iter_foreign_key_impls = generate_iter_foreign_key_impls(
        fields,
        &attributes.foreign_keys,
        attributes.ancestors.as_deref(),
        &primary_key_columns,
        &table_module,
        struct_ident,
    )?;

    // Generate BuildableTable implementation with default overrides
    let mut overrides = Vec::new();
    for (col_path, value) in &attributes.struct_defaults {
        let segments: Vec<_> = col_path.segments.iter().collect();
        if segments.len() < 2 {
            return Err(syn::Error::new_spanned(
                col_path,
                "Column path in `default(...)` must be in the format `Table::Column`",
            ));
        }
        let table_ident = &segments[segments.len() - 2].ident;

        let mut found_idx = None;
        let mut ancestor_count = 0;

        if let Some(ancestors) = &attributes.ancestors {
            ancestor_count = ancestors.len();
            for (i, ancestor_path) in ancestors.iter().enumerate() {
                if let Some(last_segment) = ancestor_path.segments.last()
                    && last_segment.ident == *table_ident
                {
                    found_idx = Some(i);
                    break;
                }
            }
        }

        if found_idx.is_none() && table_module == *table_ident {
            found_idx = Some(ancestor_count);
        }

        if found_idx.is_some() {
            overrides.push(quote! {
                {
                    use ::diesel_builders::TrySetColumn;
                    ::diesel_builders::TrySetColumn::<#col_path>::try_set_column(
                        &mut builder,
                        (#value).to_owned()
                    ).expect(concat!("Invalid default value for column ", stringify!(#col_path)));
                }
            });
        } else {
            return Err(syn::Error::new_spanned(
                col_path,
                format!("Table `{table_ident}` not found in ancestors or self"),
            ));
        }
    }

    let buildable_table_impl = quote! {
        impl ::diesel_builders::BuildableTable for #table_module::table {
            type NestedAncestorBuilders =
                <<#table_module::table as ::diesel_builders::DescendantWithSelf>::NestedAncestorsWithSelf as ::diesel_builders::NestedBundlableTables>::NestedBundleBuilders;
            type NestedCompletedAncestorBuilders =
                <<#table_module::table as ::diesel_builders::DescendantWithSelf>::NestedAncestorsWithSelf as ::diesel_builders::NestedBundlableTables>::NestedCompletedBundleBuilders;

            fn default_bundles() -> Self::NestedAncestorBuilders {
                #[allow(unused_mut)]
                let mut bundles = <Self::NestedAncestorBuilders as Default>::default();
                let mut builder = ::diesel_builders::TableBuilder::<Self>::from_bundles(bundles);
                #(#overrides)*
                builder.into_bundles()
            }
        }
    };

    // Generate final output
    Ok(quote! {
        #(#warnings)*
        #table_macro
        #typed_column_impls
        #get_column_impls
        #(#indexed_column_impls)*
        #may_get_column_impls
        #set_column_impls
        #infallible_validate_column_impls
        #descendant_impls
        #bundlable_table_impl
        #buildable_table_impl
        #(#mandatory_same_as_impls)*
        #(#discretionary_same_as_impls)*
        #(#column_horizontal_impls)*
        #(#horizontal_key_impls)*
        #(#vertical_same_as_impls)*
        #(#foreign_key_impls)*
        #(#explicit_foreign_key_impls)*
        #(#iter_foreign_key_impls)*

        // Foreign primary key implementations for triangular relations
        #(#triangular_fpk_impls)*

        // Allow tables to appear in same query with ancestors
        #(#allow_same_query_calls)*

        // Warnings
        #(#warnings)*

        // Auto-implement TableExt for the table associated with this model.
        impl ::diesel_builders::TableExt for #table_module::table {
            const TABLE_NAME: &'static str = #table_name;
            type NewRecord = #new_record;
            type NewValues = #new_record_type;
            type Model = #struct_ident;
            type NestedPrimaryKeyColumns = #nested_primary_keys;
            type Error = #error_type;

            fn default_new_values() -> Self::NewValues {
                #default_new_record
            }
        }
    })
}
