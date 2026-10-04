//! Generates `TableModel` schemas and builder implementations.

#[path = "table_model/accumulated_traits.rs"]
mod accumulated_traits;
#[path = "table_model/attribute_parsing.rs"]
mod attribute_parsing;
#[path = "table_model/foreign_keys.rs"]
mod foreign_keys;
#[path = "table_model/get_column.rs"]
mod get_column;
#[path = "table_model/may_get_columns.rs"]
mod may_get_columns;
#[path = "table_model/primary_key.rs"]
mod primary_key;
#[path = "table_model/set_columns.rs"]
mod set_columns;
#[path = "table_model/table_generation.rs"]
mod table_generation;
#[path = "table_model/take_column.rs"]
mod take_column;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
#[path = "table_model/typed_column.rs"]
mod typed_column;
#[path = "table_model/validate_record.rs"]
mod validate_record;
#[path = "table_model/vertical_same_as.rs"]
mod vertical_same_as;

use std::collections::HashMap;

use accumulated_traits::generate_accumulated_traits;
use attribute_parsing::{
    extract_diesel_attributes, extract_discretionary_table, extract_field_default_value,
    extract_mandatory_table, extract_same_as_columns, extract_table_model_attributes,
    is_field_discretionary, is_field_infallible, is_field_mandatory, validate_field_attributes,
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
use take_column::generate_take_column_impls;
use typed_column::generate_typed_column_impls;
use validate_record::generate_infallible_validate_record_impl;
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
    /// Columns with an explicit default for the `DefaultColumns` group.
    default_columns: Vec<syn::Path>,
    /// Empty-state values for the new record tuple.
    empty_values: Vec<proc_macro2::TokenStream>,
    /// Warnings to be emitted.
    warnings: Vec<proc_macro2::TokenStream>,
}

/// Process fields to extract columns, validation status, and default values.
fn process_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    table_module: &syn::Ident,
    primary_key_columns: &[syn::Ident],
    attributes: &attribute_parsing::TableModelAttributes,
) -> syn::Result<ProcessedFields> {
    let mut new_record_columns = Vec::new();
    let mut infallible_records = Vec::new();
    let mut default_values = Vec::new();
    let mut default_columns = Vec::new();
    let mut empty_values = Vec::new();
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
            warnings.push(redundant_infallible_warning(field, field_name));
        }

        if is_field_infallible(field) || attributes.error.is_none() {
            infallible_records.push(syn::parse_quote!(#table_module::#field_name));
        }

        default_values.push(field_default_value(field));
        if extract_field_default_value(field).is_some() {
            default_columns.push(syn::parse_quote!(#table_module::#field_name));
        }

        empty_values.push(field_empty_value(field));
    }

    Ok(ProcessedFields {
        new_record_columns,
        infallible_records,
        default_values,
        default_columns,
        empty_values,
        warnings,
    })
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

/// Generates a positional `SameAsIndex` implementation for each triangular
/// column, mapping the column to its `typenum` tuple index.
fn same_as_index_impls(
    columns: &[syn::Type],
    same_as_trait: &proc_macro2::TokenStream,
) -> Vec<proc_macro2::TokenStream> {
    columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let idx = crate::utils::typenum_ident(index);
            quote! {
                impl ::diesel_builders::#same_as_trait for #column {
                    type Idx = ::diesel_builders::typenum::#idx;
                }
            }
        })
        .collect()
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
#[expect(
    clippy::too_many_lines,
    reason = "orchestration entry point that parses the input then wires each TableModel codegen phase"
)]
pub fn derive_table_model_impl(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;

    // Parse attributes
    let (table_module_opt, primary_key_columns) = extract_diesel_attributes(input)?;
    let attributes = extract_table_model_attributes(input)?;

    let table_module = if let Some(module) = table_module_opt {
        module
    } else {
        use syn::ext::IdentExt;
        let struct_name = struct_ident.unraw().to_string();
        let table_name_str = format!("{}s", crate::utils::camel_to_snake_case(&struct_name));
        let mut table_ident = syn::parse_str::<syn::Ident>(&table_name_str)
            .or_else(|_| syn::parse_str::<syn::Ident>(&format!("r#{table_name_str}")))?;
        table_ident.set_span(struct_ident.span());
        table_ident
    };

    if let Some(ancestors) = &attributes.ancestors {
        if ancestors.is_empty() {
            return Err(syn::Error::new_spanned(
                input,
                "`#[table_model(ancestors(...))]` must list at least one ancestor table",
            ));
        }
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
    let accumulated_traits_impls = generate_accumulated_traits(
        fields,
        &table_module,
        struct_ident,
        &primary_key_columns,
        attributes.surrogate_key,
        attributes.error.is_some(),
    );
    let indexed_column_impls = generate_indexed_column_impls(&table_module, &primary_key_columns);
    let nested_primary_keys = format_as_nested_tuple(
        primary_key_columns.iter().map(|col| quote::quote! { #table_module::#col }),
    );

    let ProcessedFields {
        new_record_columns,
        infallible_records,
        default_values,
        default_columns,
        empty_values,
        warnings,
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

    // Validate mandatory triangular relations on primary keys
    for field in fields {
        if is_field_mandatory(field)
            && let Some(mandatory_table) = extract_mandatory_table(field)?
        {
            // Check if ALL primary key columns have a same_as pointing to this
            // mandatory table
            for pk_col_name in &primary_key_columns {
                let pk_field = fields.iter().find(|f| f.ident.as_ref() == Some(pk_col_name));

                if let Some(pk_field) = pk_field {
                    let same_as_cols_groups = extract_same_as_columns(pk_field)?;

                    let mandatory_table_ident = crate::utils::last_segment_ident(&mandatory_table)?;
                    let mut has_same_as_to_mandatory = false;
                    for (index, path) in
                        same_as_cols_groups.iter().flat_map(|group| group.iter().enumerate())
                    {
                        let number_of_segments = path.segments.len();
                        if number_of_segments < 2 {
                            // Bare suffix paths identify relation keys.
                            if index > 0 {
                                continue;
                            }
                            return Err(syn::Error::new_spanned(
                                path,
                                "Column path in `#[same_as(...)]` must be in the format `table::column`",
                            ));
                        }
                        if path
                            .segments
                            .iter()
                            .rev()
                            .nth(1)
                            .is_some_and(|segment| segment.ident == *mandatory_table_ident)
                        {
                            has_same_as_to_mandatory = true;
                            break;
                        }
                    }

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

    let joinable_impls = attributes.ancestors.iter().flatten().map(|ancestor| {
        if let [pk] = primary_key_columns.as_slice() {
            quote! {
                ::diesel::joinable!(#table_module -> #ancestor (#pk));
            }
        } else {
            quote! {
                impl ::diesel::query_source::JoinTo<#ancestor::table> for #table_module::table {
                    type FromClause = #ancestor::table;
                    type OnClause = <<
                        #table_module::table as ::diesel::Table
                    >::PrimaryKey as ::diesel::expression_methods::EqAll<
                        <#ancestor::table as ::diesel::Table>::PrimaryKey
                    >>::Output;

                    fn join_target(rhs: #ancestor::table) -> (Self::FromClause, Self::OnClause) {
                        (rhs, ::diesel::expression_methods::EqAll::eq_all(
                            ::diesel::Table::primary_key(&#table_module::table),
                            ::diesel::Table::primary_key(&rhs),
                        ))
                    }
                }

                impl ::diesel::query_source::JoinTo<#table_module::table> for #ancestor::table {
                    type FromClause = #table_module::table;
                    type OnClause = <<
                        #ancestor::table as ::diesel::Table
                    >::PrimaryKey as ::diesel::expression_methods::EqAll<
                        <#table_module::table as ::diesel::Table>::PrimaryKey
                    >>::Output;

                    fn join_target(rhs: #table_module::table) -> (Self::FromClause, Self::OnClause) {
                        (rhs, ::diesel::expression_methods::EqAll::eq_all(
                            ::diesel::Table::primary_key(&#ancestor::table),
                            ::diesel::Table::primary_key(&rhs),
                        ))
                    }
                }
            }
        }
    }).collect::<Vec<_>>();

    let table_name = table_module.to_string();

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

    let take_column_impls = generate_take_column_impls(&new_record_columns, &table_module);

    let error_type = attributes
        .error
        .as_ref()
        .map(|t| quote::quote! { #t })
        .unwrap_or(quote::quote! { std::convert::Infallible });

    let record_error_type = attributes
        .record_error
        .as_ref()
        .map(|t| quote::quote! { #t })
        .unwrap_or(quote::quote! { std::convert::Infallible });

    let default_columns_tuple = format_as_nested_tuple(&default_columns);
    let empty_new_record = format_as_nested_tuple(&empty_values);

    // Tables without a declared record error type get the infallible
    // whole-record validation; otherwise the model supplies it.
    let validate_record_impl = if attributes.record_error.is_none() {
        generate_infallible_validate_record_impl(&table_module)
    } else {
        TokenStream::new()
    };

    // Generate Root/Descendant implementations
    // If ancestors are specified, generate Descendant; otherwise generate Root
    let descendant_impls = if let Some(ref ancestors) = attributes.ancestors {
        let table_type: syn::Type = syn::parse_quote!(#table_module::table);
        // Convert ancestor module paths to table types for the trait
        // implementation
        let ancestor_tables: Vec<syn::Type> =
            ancestors.iter().map(|a| syn::parse_quote!(#a::table)).collect();
        let nested_ancestors = format_as_nested_tuple(&ancestor_tables);
        let Some(root) = ancestor_tables.first() else {
            return Err(syn::Error::new_spanned(
                input,
                "`#[table_model(ancestors(...))]` must list at least one ancestor table",
            ));
        };
        let aux_impls =
            crate::descendant::generate_auxiliary_descendant_impls(&table_type, &ancestor_tables);

        quote! {
            impl ::diesel_builders::Descendant for #table_type {
                type NestedAncestors = #nested_ancestors;
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
                type NestedAncestors = ();
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

    // Generate the positional `SameAsIndex` implementations for the mandatory
    // and discretionary triangular columns.
    let mandatory_same_as_impls =
        same_as_index_impls(&mandatory_columns, &quote!(MandatorySameAsIndex));
    let discretionary_same_as_impls =
        same_as_index_impls(&discretionary_columns, &quote!(DiscretionarySameAsIndex));

    let (horizontal_key_impls, column_horizontal_impls) =
        generate_horizontal_key_impls(fields, &table_module)?;

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

    let buildable_table_impl = generate_buildable_table_impl(&table_module, &attributes)?;

    let model_upsert_impl = generate_model_upsert_impl(struct_ident, &table_module);

    // Generate final output
    Ok(quote! {
        #(#warnings)*
        #table_macro
        #typed_column_impls
        #get_column_impls
        #accumulated_traits_impls
        #(#indexed_column_impls)*
        #may_get_column_impls
        #set_column_impls
        #take_column_impls
        #validate_record_impl
        #infallible_validate_column_impls
        #descendant_impls
        #bundlable_table_impl
        #buildable_table_impl
        #model_upsert_impl
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

        #(#joinable_impls)*


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
            type DefaultColumns = #default_columns_tuple;
            type RecordError = #record_error_type;

            fn default_new_values() -> Self::NewValues {
                #default_new_record
            }

            fn empty_new_values() -> Self::NewValues {
                #empty_new_record
            }
        }
    })
}

/// Generates the `ModelUpsert` implementation for the model.
///
/// The upsert statement chains diesel's inherent `on_conflict`, `do_update`,
/// and `set` methods, whose trait bounds reference `diesel::internal` items and
/// so cannot be named in a generic `where` clause. Emitting the impl where the
/// table type is concrete lets the compiler discharge those bounds itself, so
/// the only free parameter left is the connection.
fn generate_model_upsert_impl(struct_ident: &Ident, table_module: &Ident) -> TokenStream {
    let model_table_type = quote! { #table_module::table };
    let upsert_nested_columns = quote! {
        <<#model_table_type as ::diesel::Table>::AllColumns as ::diesel_builders::tuplities::NestTuple>::Nested
    };
    let upsert_changeset = quote! {
        <<#upsert_nested_columns as ::diesel_builders::columns::TupleEqAll>::EqAll as ::diesel_builders::tuplities::FlattenNestedTuple>::Flattened
    };
    let upsert_statement = quote! {
        ::diesel::dsl::Set<
            ::diesel::dsl::DoUpdate<
                ::diesel::dsl::OnConflict<
                    ::diesel::query_builder::InsertStatement<
                        #model_table_type,
                        <#upsert_changeset as ::diesel::Insertable<#model_table_type>>::Values,
                    >,
                    <#model_table_type as ::diesel::Table>::PrimaryKey,
                >,
            >,
            #upsert_changeset,
        >
    };
    quote! {
        impl<Conn> ::diesel_builders::ModelUpsert<Conn> for #struct_ident
        where
            Conn: ::diesel::connection::LoadConnection,
            for<'query> #upsert_statement: ::diesel::query_dsl::methods::LoadQuery<
                'query,
                Conn,
                <#model_table_type as ::diesel_builders::TableExt>::Model,
            >,
        {
            fn upsert(
                &self,
                conn: &mut Conn,
            ) -> ::diesel::QueryResult<<#model_table_type as ::diesel_builders::TableExt>::Model>
            where
                Self: Sized,
            {
                use ::diesel::{RunQueryDsl, Table};
                let table: #model_table_type = ::core::default::Default::default();
                let columns = <#upsert_nested_columns as ::core::default::Default>::default();
                let values = ::diesel_builders::tuplities::FlattenNestedTuple::flatten(
                    ::diesel_builders::columns::TupleEqAll::eq_all(
                        columns,
                        ::diesel_builders::GetNestedColumns::<#upsert_nested_columns>::get_nested_columns(self),
                    ),
                );
                let changes = ::diesel_builders::tuplities::FlattenNestedTuple::flatten(
                    ::diesel_builders::columns::TupleEqAll::eq_all(
                        columns,
                        ::diesel_builders::GetNestedColumns::<#upsert_nested_columns>::get_nested_columns(self),
                    ),
                );
                ::diesel::insert_into(table)
                    .values(values)
                    .on_conflict(table.primary_key())
                    .do_update()
                    .set(changes)
                    .get_result(conn)
            }
        }
    }
}

/// Generates the `BuildableTable` implementation.
///
/// Struct default overrides are applied as raw sets on the staged default
/// builder, before any validation. `DefaultError` composes one stage for the
/// base defaults and one stage per override, covering the overridden column
/// and its vertical same-as group.
#[expect(
    clippy::too_many_lines,
    reason = "Default validation stages and constructor generation share one ordered pipeline."
)]
fn generate_buildable_table_impl(
    table_module: &Ident,
    attributes: &attribute_parsing::TableModelAttributes,
) -> syn::Result<TokenStream> {
    let mut overrides: Vec<(&syn::Path, &syn::Expr)> = Vec::new();
    for (col_path, value) in &attributes.struct_defaults {
        let table_ident =
            col_path.segments.iter().rev().nth(1).map(|segment| &segment.ident).ok_or_else(
                || {
                    syn::Error::new_spanned(
                        col_path,
                        "Column path in `default(...)` must be in the format `Table::Column`",
                    )
                },
            )?;

        let in_ancestors = attributes.ancestors.as_deref().is_some_and(|ancestors| {
            ancestors.iter().any(|ancestor_path| {
                ancestor_path
                    .segments
                    .last()
                    .is_some_and(|last_segment| last_segment.ident == *table_ident)
            })
        });

        if !in_ancestors && *table_module != *table_ident {
            return Err(syn::Error::new_spanned(
                col_path,
                format!("Table `{table_ident}` not found in ancestors or self"),
            ));
        }

        overrides.push((col_path, value));
    }

    let builder_type = quote! { ::diesel_builders::DefaultBuilder<Self> };

    // Validation stages: the base default stage, then one stage per struct
    // default override covering the overridden column and its vertical
    // same-as group.
    let mut stage_errors = vec![quote! {
        <Self::NestedDefaultBundles as ::diesel_builders::BuildDefaults>::Error
    }];
    let mut check_statements = Vec::new();
    let stage_count = 1 + overrides.len();

    for (i, (col_path, _)) in overrides.iter().enumerate() {
        let override_group = quote! {
            <<#col_path as ::diesel_builders::VerticalSameAsGroup>::VerticalSameAsNestedColumns
                as ::diesel_builders::tuplities::NestedTuplePushFront<#col_path>>::Output
        };

        stage_errors.push(quote! {
            <#builder_type as ::diesel_builders::CheckAndMoveColumns<#override_group>>::Error
        });

        let mapping = stage_error_mapping(i + 1, stage_count);
        check_statements.push(quote! {
            <#builder_type as ::diesel_builders::CheckAndMoveColumns<#override_group>>::check_and_move_columns(
                &mut builder,
            )#mapping?;
        });
    }

    let default_error = composed_stage_errors(stage_errors);

    let raw_sets: Vec<TokenStream> = overrides
        .iter()
        .map(|(col_path, value)| {
            quote! {
                <#builder_type as ::diesel_builders::SetColumn<#col_path>>::set_column(
                    &mut builder,
                    (#value).to_owned(),
                );
            }
        })
        .collect();

    let base_mapping = stage_error_mapping(0, stage_count);

    let ancestor_empties: Vec<TokenStream> = attributes
        .ancestors
        .iter()
        .flatten()
        .map(|ancestor| {
            quote! {
                ::diesel_builders::TableBuilderBundle::<#ancestor::table>::empty()
            }
        })
        .collect();
    let empty_bundles = ancestor_empties.iter().rev().fold(
        quote! { (::diesel_builders::TableBuilderBundle::<#table_module::table>::empty(),) },
        |tail, head| quote! { (#head, #tail) },
    );

    Ok(quote! {
        impl ::diesel_builders::BuildableTable for #table_module::table {
            type NestedAncestorBuilders =
                <<#table_module::table as ::diesel_builders::DescendantWithSelf>::NestedAncestorsWithSelf as ::diesel_builders::NestedBundlableTables>::NestedBundleBuilders;
            type NestedCompletedAncestorBuilders =
                <<#table_module::table as ::diesel_builders::DescendantWithSelf>::NestedAncestorsWithSelf as ::diesel_builders::NestedBundlableTables>::NestedCompletedBundleBuilders;
            type NestedDefaultBundles =
                <<#table_module::table as ::diesel_builders::DescendantWithSelf>::NestedAncestorsWithSelf as ::diesel_builders::NestedBundlableTables>::NestedDefaultBundleBuilders;
            type DefaultError = #default_error;

            fn try_builder() -> core::result::Result<
                ::diesel_builders::TableBuilder<Self>,
                Self::DefaultError,
            > {
                let mut builder = ::diesel_builders::DefaultBuilder::<Self>::new();
                #(#raw_sets)*
                <#builder_type as ::diesel_builders::BuildDefaults>::check_defaults(
                    &mut builder,
                )#base_mapping?;
                #(#check_statements)*
                Ok(<#builder_type as ::diesel_builders::BuildDefaults>::into_checked(builder))
            }

            fn empty_builder() -> ::diesel_builders::TableBuilder<Self> {
                ::diesel_builders::TableBuilder::from_bundles(#empty_bundles)
            }
        }
    })
}

/// Builds the `map_err` expression placing a stage error at its position in
/// the right-nested `EitherValidationError` tree over `stages` stages: the
/// stage at depth `k` is reached by `k` `Right` steps, plus one `Left` step
/// unless it is the rightmost stage. A single stage maps directly.
fn stage_error_mapping(stage: usize, stages: usize) -> TokenStream {
    if stages == 1 {
        return TokenStream::new();
    }

    let error_ident = syn::Ident::new("err", proc_macro2::Span::call_site());
    let core = if stage == stages - 1 {
        quote! { #error_ident }
    } else {
        quote! {
            ::diesel_builders::EitherValidationError::Left(#error_ident)
        }
    };
    let mut mapping = core;
    for _ in 0..stage {
        mapping = quote! {
            ::diesel_builders::EitherValidationError::Right(#mapping)
        };
    }

    quote! { .map_err(|#error_ident| #mapping) }
}

/// Builds the `DefaultError` type over the validation stage errors. A single
/// stage retains its own error type, and multiple stages compose as a
/// right-nested `EitherValidationError`.
fn composed_stage_errors(mut stage_errors: Vec<TokenStream>) -> TokenStream {
    let Some(last) = stage_errors.pop() else {
        return quote! { std::convert::Infallible };
    };

    let mut composed = last;
    for type_expr in stage_errors.iter().rev() {
        composed = quote! {
            ::diesel_builders::EitherValidationError<#type_expr, #composed>
        };
    }
    composed
}

/// Computes the horizontal (triangular) same-as `HorizontalKey` and
/// `HorizontalSameAsGroup` implementations for the table's fields.
#[expect(
    clippy::too_many_lines,
    reason = "one cohesive pass that resolves same-as attributes into horizontal keys whose map borrows through the field list"
)]
fn generate_horizontal_key_impls(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::Token![,]>,
    table_module: &Ident,
) -> syn::Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    // Collect Horizontal Keys
    // Map from TargetTable (last segment ident) to list of (KeyField,
    // IsMandatory, TargetTablePath)
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
                    // If this is the explicit key, skip it (it's not a target
                    // column)
                    if let Some(k) = &explicit_key_ident
                        && i == 1
                        && col_path.segments.last().map(|s| &s.ident) == Some(k)
                    {
                        continue;
                    }

                    let table_ident = col_path
                        .segments
                        .iter()
                        .rev()
                        .nth(1)
                        .map(|segment| &segment.ident)
                        .ok_or_else(|| {
                            syn::Error::new_spanned(
                                col_path,
                                "Non-key column path in #[same_as(...)] must be in the format `table::column` or a `#[mandatory]`/`#[discretionary]` attribute is missing.",
                            )
                        })?;

                    // Check if this matches a target table
                    if let Some(keys) = potential_keys.get(table_ident) {
                        // Found a match for target table

                        let selected_key: Option<Ident> = if let Some(k_ident) = &explicit_key_ident
                        {
                            // Verify the explicit key belongs to this target
                            // table
                            if keys.iter().any(|(kf, _, _)| kf == &k_ident) {
                                Some(k_ident.clone())
                            } else {
                                // The explicit key belongs to a different
                                // target table.
                                None
                            }
                        } else if keys.len() == 1 {
                            let (key, _, _) = &keys[0];
                            Some((*key).clone())
                        } else {
                            // Ambiguous
                            let available_keys: Vec<String> =
                                keys.iter().map(|(k, _, _)| format!("`{k}`")).collect();
                            let available_keys_str = available_keys.join(", ");
                            let col_name = crate::utils::last_segment_ident(col_path)?;

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
    // We do not filter out keys with no columns, as they still need to
    // implement HorizontalKey to satisfy BundlableTable bounds, even if
    // they don't propagate any values.

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

        // We check that the key has at least one host and one foreign column,
        // or raise an appropriate compile-time error.
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
                let idx_ident = crate::utils::typenum_ident(i);
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
    Ok((horizontal_key_impls, column_horizontal_impls))
}

/// Computes the default-value expression for a field's slot in the new-record
/// tuple: a user default if present, `Some(None)` for nullable columns, or
/// `None` otherwise.
fn field_default_value(field: &syn::Field) -> TokenStream {
    let user_default = extract_field_default_value(field);
    let is_nullable = is_option(&field.ty);
    if let Some(def) = user_default {
        quote::quote! { Some((#def).to_owned().into()) }
    } else if is_nullable {
        quote::quote! { Some(None) }
    } else {
        quote::quote! { None }
    }
}

/// Computes the empty-state value expression for a field's slot in the
/// new-record tuple: `Some(None)` for nullable columns and `None` otherwise,
/// without applying any explicit defaults.
fn field_empty_value(field: &syn::Field) -> TokenStream {
    if is_option(&field.ty) {
        quote::quote! { Some(None) }
    } else {
        quote::quote! { None }
    }
}

/// Builds a compile-time deprecation warning for a field marked `#[infallible]`
/// on a model with no error type, where the attribute is redundant. The warning
/// is anchored to the `#[infallible]` attribute when it can be located.
fn redundant_infallible_warning(field: &syn::Field, field_name: &Ident) -> TokenStream {
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

    let const_name = syn::Ident::new(&format!("__WARN_REDUNDANT_INFALLIBLE_{field_name}"), span);
    quote! {
        const _: () = {
            #[deprecated(note = #warning_msg)]
            #[allow(non_upper_case_globals)]
            const #const_name: () = ();
            let _ = #const_name;
        };
    }
}
