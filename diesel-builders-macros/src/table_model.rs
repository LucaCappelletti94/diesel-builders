//! Module for `TableModel` derive macro implementation.
//!
//! This module contains the implementation of the `TableModel` derive macro,
//! split into logical components for better maintainability.

mod attribute_parsing;
mod get_column;
mod may_get_columns;
mod primary_key;
mod set_columns;
mod table_generation;
mod typed_column;

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use attribute_parsing::{
    extract_field_default_value, extract_primary_key_columns, extract_table_model_attributes,
    extract_table_module, is_field_discretionary, is_field_infallible, is_field_mandatory,
    validate_field_attributes,
};

use get_column::generate_get_column_impls;
use primary_key::generate_indexed_column_impls;
use table_generation::generate_table_macro;
use typed_column::generate_typed_column_impls;

use crate::utils::{format_as_nested_tuple, is_option};

/// Struct to hold processed field information.
struct ProcessedFields {
    /// Columns for the new record tuple.
    new_record_columns: Vec<syn::Path>,
    /// Records that can fail validation (index, path).
    fallible_records: Vec<(usize, syn::Path)>,
    /// Records that are infallible (index, path).
    infallible_records: Vec<(usize, syn::Path)>,
    /// Default values for fields.
    default_values: Vec<proc_macro2::TokenStream>,
}

/// Process fields to extract columns, validation status, and default values.
fn process_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
    primary_key_columns: &[syn::Ident],
    attributes: &attribute_parsing::TableModelAttributes,
) -> syn::Result<ProcessedFields> {
    let mut new_record_columns = Vec::new();
    let mut fallible_records = Vec::new();
    let mut infallible_records = Vec::new();
    let mut default_values = Vec::new();
    let mut idx = 0;

    for field in fields {
        validate_field_attributes(field)?;

        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "Field must have a name"))?;

        // Check if field is a primary key
        let is_pk = primary_key_columns.iter().any(|pk| pk == field_name);

        if is_pk {
            if extract_field_default_value(field).is_some() {
                return Err(syn::Error::new_spanned(
                    field,
                    "Primary key cannot have a `default` value",
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
        if is_field_infallible(field) || attributes.error.is_none() {
            infallible_records.push((idx, syn::parse_quote!(#table_module::#field_name)));
        } else {
            fallible_records.push((idx, syn::parse_quote!(#table_module::#field_name)));
        }

        // Default value logic
        let user_default = extract_field_default_value(field);
        let is_nullable = is_option(&field.ty);

        let default_val = if let Some(def) = user_default {
            quote::quote! { Some((#def).into()) }
        } else if is_nullable {
            quote::quote! { Some(None) }
        } else {
            quote::quote! { None }
        };
        default_values.push(default_val);

        idx += 1;
    }

    Ok(ProcessedFields {
        new_record_columns,
        fallible_records,
        infallible_records,
        default_values,
    })
}

/// Collect mandatory and discretionary triangular relation columns.
fn collect_triangular_columns(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
) -> (Vec<syn::Type>, Vec<syn::Type>) {
    let mut mandatory_columns = Vec::new();
    let mut discretionary_columns = Vec::new();

    for field in fields {
        if let Some(field_name) = &field.ident {
            if is_field_mandatory(field) {
                mandatory_columns.push(syn::parse_quote!(#table_module::#field_name));
            } else if is_field_discretionary(field) {
                discretionary_columns.push(syn::parse_quote!(#table_module::#field_name));
            }
        }
    }

    (mandatory_columns, discretionary_columns)
}

/// Main entry point for the `TableModel` derive macro.
#[allow(clippy::too_many_lines)]
pub fn derive_table_model_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;

    // Parse attributes
    let table_module_opt = extract_table_module(input);
    let primary_key_columns = extract_primary_key_columns(input);
    let attributes = extract_table_model_attributes(input);

    let table_module = if let Some(module) = table_module_opt {
        module
    } else {
        let struct_name = struct_ident.to_string();
        let table_name_str = format!("{}s", crate::utils::camel_to_snake_case(&struct_name));
        syn::Ident::new(&table_name_str, struct_ident.span())
    };

    if let Some(ancestors) = &attributes.ancestors {
        let table_type_str = table_module.to_string();

        let mut seen = std::collections::HashSet::new();
        for ancestor in ancestors {
            let ancestor_str = quote::quote!(#ancestor).to_string().replace(' ', "");

            if ancestor_str == table_type_str {
                return Err(syn::Error::new_spanned(
                    ancestor,
                    "Table cannot be its own `ancestor`",
                ));
            }

            if !seen.insert(ancestor_str) {
                return Err(syn::Error::new_spanned(
                    ancestor,
                    "Duplicate `ancestor` in hierarchy",
                ));
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

    // Validate that all primary key columns exist in the struct
    let field_names: std::collections::HashSet<_> =
        fields.iter().filter_map(|f| f.ident.as_ref()).collect();

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

    // Validate fields before generation to ensure unsupported attributes are reported correctly
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
        primary_key_columns
            .iter()
            .map(|col| quote::quote! { #table_module::#col }),
    );

    let ProcessedFields {
        new_record_columns,
        fallible_records,
        infallible_records,
        default_values,
    } = process_fields(fields, &table_module, &primary_key_columns, &attributes)?;

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

    let new_record = format_as_nested_tuple(&new_record_columns);
    let default_new_record = format_as_nested_tuple(&default_values);
    let new_record_type = format_as_nested_tuple(
        new_record_columns
            .iter()
            .map(|col| quote::quote! { Option<<#col as diesel_builders::Typed>::Type> }),
    );
    let may_get_column_impls =
        may_get_columns::generate_may_get_column_impls(&new_record_columns, &table_module);

    let set_column_impls =
        set_columns::generate_set_column_impls(&infallible_records, &table_module);

    let set_column_unchecked_impls =
        set_columns::generate_set_column_unchecked_traits(&fallible_records, &table_module);

    let error_type = attributes
        .error
        .map(|t| quote::quote! { #t })
        .unwrap_or(quote::quote! { std::convert::Infallible });

    // Generate Root/Descendant implementations
    // If ancestors are specified, generate Descendant; otherwise generate Root
    let descendant_impls = if let Some(ref ancestors) = attributes.ancestors {
        let root = attributes
            .root
            .or_else(|| ancestors.first().map(|a| syn::parse_quote!(#a::table)));

        if let Some(root) = root {
            let table_type: syn::Type = syn::parse_quote!(#table_module::table);
            // Convert ancestor module paths to table types for the trait implementation
            let ancestor_tables: Vec<syn::Type> = ancestors
                .iter()
                .map(|a| syn::parse_quote!(#a::table))
                .collect();
            let aux_impls = crate::descendant::generate_auxiliary_descendant_impls(
                &table_type,
                &ancestor_tables,
                &root,
            );

            quote! {
                impl diesel_builders::Descendant for #table_type {
                    type Ancestors = (#(#ancestor_tables,)*);
                    type Root = #root;
                }
                #aux_impls
            }
        } else {
            syn::Error::new_spanned(
                input,
                "`ancestors` attribute provided but no `root` could be inferred (ancestors list is empty and no root specified)",
            )
            .to_compile_error()
        }
    } else {
        // No ancestors attribute means this is a root table
        let table_type: syn::Type = syn::parse_quote!(#table_module::table);
        let aux_impls =
            crate::descendant::generate_auxiliary_descendant_impls(&table_type, &[], &table_type);

        quote! {
            impl diesel_builders::Root for #table_type {}

            impl diesel_builders::Descendant for #table_type {
                type Ancestors = ();
                type Root = Self;
            }

            #aux_impls
        }
    };

    // Generate ForeignPrimaryKey implementation for single-column primary key with single ancestor
    let foreign_primary_key_impl = if let Some(ancestors) = &attributes.ancestors {
        if ancestors.len() == 1 && primary_key_columns.len() == 1 {
            let ancestor = &ancestors[0];
            let pk_column = &primary_key_columns[0];

            // Use the fpk! macro to generate the implementation
            // ancestors are stored as module paths (e.g., parent_table) without ::table suffix
            let fpk_tokens: proc_macro::TokenStream = crate::fpk(
                quote! {
                    #table_module::#pk_column -> #ancestor
                }
                .into(),
            );

            proc_macro2::TokenStream::from(fpk_tokens)
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    // Always generate BundlableTable implementation
    let mandatory_tuple = if mandatory_columns.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#mandatory_columns,)*) }
    };
    let discretionary_tuple = if discretionary_columns.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#discretionary_columns,)*) }
    };

    let bundlable_table_impl = quote! {
        impl diesel_builders::BundlableTable for #table_module::table {
            type MandatoryTriangularColumns = #mandatory_tuple;
            type DiscretionaryTriangularColumns = #discretionary_tuple;
        }
    };

    // Generate MandatorySameAsIndex implementations for mandatory columns
    let mandatory_same_as_impls: Vec<_> = mandatory_columns
        .iter()
        .enumerate()
        .map(|(i, column)| {
            let idx = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            quote! {
                impl diesel_builders::MandatorySameAsIndex for #column {
                    type Idx = diesel_builders::typenum::#idx;
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
                impl diesel_builders::DiscretionarySameAsIndex for #column {
                    type Idx = diesel_builders::typenum::#idx;
                }
            }
        })
        .collect();

    // Generate HorizontalSameAsGroup for each column (with empty tuples)
    // Skip the primary key column if it's a single-column primary key to avoid conflicts
    let column_horizontal_impls: Vec<_> = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            // Skip if this is the only primary key column (to avoid conflict with PrimaryKey impl)
            if primary_key_columns.len() == 1 && primary_key_columns[0] == *field_name {
                return None;
            }
            Some(quote! {
                impl diesel_builders::HorizontalSameAsGroup for #table_module::#field_name {
                    type Idx = diesel_builders::typenum::U0;
                    type MandatoryHorizontalKeys = ();
                    type DiscretionaryHorizontalKeys = ();
                }
            })
        })
        .collect();

    // Generate HorizontalSameAsGroup for the primary key (with triangular columns)
    // Only generate if the primary key is a single column (not a tuple/composite key)
    // to avoid orphan rule violations
    let horizontal_same_as_group_impl = if primary_key_columns.len() == 1 {
        quote! {
            impl diesel_builders::HorizontalSameAsGroup for <#table_module::table as diesel::Table>::PrimaryKey {
                type Idx = diesel_builders::typenum::U0;
                type MandatoryHorizontalKeys = (#(#mandatory_columns,)*);
                type DiscretionaryHorizontalKeys = (#(#discretionary_columns,)*);
            }
        }
    } else {
        quote! {}
    };

    // Generate final output
    Ok(quote! {
        #table_macro
        #typed_column_impls
        #get_column_impls
        #(#indexed_column_impls)*
        #may_get_column_impls
        #set_column_impls
        #set_column_unchecked_impls
        #descendant_impls
        #foreign_primary_key_impl
        #bundlable_table_impl
        #(#mandatory_same_as_impls)*
        #(#discretionary_same_as_impls)*
        #(#column_horizontal_impls)*
        #horizontal_same_as_group_impl

        // Auto-implement TableExt for the table associated with this model.
        impl diesel_builders::TableExt for #table_module::table {
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
