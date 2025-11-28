//! Procedural macros for diesel-builders workspace.
//!
//! This crate provides attribute macros that generate trait implementations
//! for tuples, replacing the complex `macro_rules!` patterns with cleaner
//! procedural macros.

mod impl_generators;
mod tuple_generator;

use proc_macro::TokenStream;

/// Generate `DefaultTuple` trait implementations for all tuple sizes (1-32).
///
/// # Usage
///
/// Add this attribute at the module or crate level:
///
/// ```ignore
/// use diesel_builders_macros::impl_default_tuple;
///
/// #[impl_default_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
///
/// This will generate implementations of the `DefaultTuple` trait for
/// tuples of size 1 through 32.
#[proc_macro_attribute]
pub fn impl_default_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_default_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `OptionTuple` and `TransposeOptionTuple` trait implementations
/// for all tuple sizes (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_option_tuple;
///
/// #[impl_option_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_option_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_option_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `RefTuple` trait implementations for all tuple sizes (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_ref_tuple;
///
/// #[impl_ref_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_ref_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_ref_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `ClonableTuple` trait implementations for all tuple sizes (0-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_clonable_tuple;
///
/// #[impl_clonable_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_clonable_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_clonable_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `DebuggableTuple` trait implementations for all tuple sizes (0-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_debuggable_tuple;
///
/// #[impl_debuggable_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_debuggable_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_debuggable_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `Columns`, `Projection`, and `HomogeneousColumns` trait
/// implementations for all tuple sizes (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_columns;
///
/// #[impl_columns]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_columns();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `Tables`, `TableModels`, and `InsertableTableModels` trait
/// implementations for all tuple sizes (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_tables;
///
/// #[impl_tables]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_tables(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_tables();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate column getter/setter trait implementations for all tuple sizes
/// (1-32).
///
/// Generates implementations for:
/// - `GetColumns`
/// - `MayGetColumns`
/// - `SetColumns`
/// - `SetInsertableTableModelHomogeneousColumn`
/// - `TrySetColumns`
/// - `TrySetInsertableTableModelHomogeneousColumn`
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_get_columns;
///
/// #[impl_get_columns]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_get_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_get_columns();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `NestedInsertTuple` trait implementations for all tuple sizes
/// (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_nested_insert_tuple;
///
/// #[impl_nested_insert_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_nested_insert_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_nested_insert_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `NestedInsertOptionTuple` trait implementations for all tuple sizes
/// (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_nested_insert_option_tuple;
///
/// #[impl_nested_insert_option_tuple]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_nested_insert_option_tuple(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_nested_insert_option_tuple();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `BuildableTables` trait implementations for all tuple sizes (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_buildable_tables;
///
/// #[impl_buildable_tables]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_buildable_tables(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_buildable_tables();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `BundlableTables` trait implementations for all tuple sizes (1-32).
#[proc_macro_attribute]
pub fn impl_bundlable_tables(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_bundlable_tables();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `BuildableColumns` trait implementations for all tuple sizes
/// (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_buildable_columns;
///
/// #[impl_buildable_columns]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_buildable_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_buildable_columns();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `NonCompositePrimaryKeyTableModels` and `MayGetPrimaryKeys` trait
/// implementations for all tuple sizes (1-32).
///
/// Generates implementations for:
/// - `NonCompositePrimaryKeyTableModels` for tuples of models
/// - `MayGetPrimaryKeys` for tuples of optional models
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_table_model;
///
/// #[impl_table_model]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_table_model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_table_model();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `BuilderBundles` trait implementations for all tuple sizes (1-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_builder_bundles;
///
/// #[impl_builder_bundles]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_builder_bundles(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_builder_bundles();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `AncestorsOf` trait implementations for all tuple sizes (0-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_ancestors_of;
///
/// #[impl_ancestors_of]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_ancestors_of(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_ancestors_of();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `HorizontalSameAsKeys` trait implementations for all tuple sizes
/// (0-32).
///
/// # Usage
///
/// ```ignore
/// use diesel_builders_macros::impl_horizontal_same_as_keys;
///
/// #[impl_horizontal_same_as_keys]
/// mod my_module {
///     // Your code here
/// }
/// ```
#[proc_macro_attribute]
pub fn impl_horizontal_same_as_keys(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_horizontal_same_as_keys();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Derive macro to automatically implement `GetColumn` for all fields of a
/// struct.
///
/// This macro generates `GetColumn` implementations for each field in the
/// struct, assuming:
/// - The struct has a `#[diesel(table_name = ...)]` attribute
/// - Each field name matches a column name in the table
/// - Each column implements `TypedColumn` trait
#[proc_macro_derive(GetColumn)]
pub fn derive_get_column(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &input.ident;

    // Find the diesel(table_name = ...) attribute
    let table_name = input.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("diesel") {
            return None;
        }

        let mut table_name = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                let value = meta.value()?;
                let lit: syn::Ident = value.parse()?;
                table_name = Some(lit);
                Ok(())
            } else {
                Ok(())
            }
        });
        table_name
    });

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "GetColumn derive requires a #[diesel(table_name = ...)] attribute",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => {
                    return syn::Error::new_spanned(
                        &input,
                        "GetColumn can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(&input, "GetColumn can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote::quote! {
            impl diesel_additions::GetColumn<#table_name::#field_name> for #struct_name {
                fn get_column(&self) -> &<#table_name::#field_name as diesel_additions::TypedColumn>::Type {
                    &self.#field_name
                }
            }
        }
    });

    quote::quote! {
        #(#impls)*
    }
    .into()
}

/// Derive macro to automatically implement `MayGetColumn` for all fields of a
/// struct.
///
/// This macro generates `MayGetColumn` implementations for each field in the
/// struct, assuming:
/// - The struct has a `#[diesel(table_name = ...)]` attribute
/// - Each field name matches a column name in the table
/// - Each field is wrapped in `Option<T>`
/// - Each column implements `TypedColumn` trait
#[proc_macro_derive(MayGetColumn)]
pub fn derive_may_get_column(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &input.ident;

    // Find the diesel(table_name = ...) attribute
    let table_name = input.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("diesel") {
            return None;
        }

        let mut table_name = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                let value = meta.value()?;
                let lit: syn::Ident = value.parse()?;
                table_name = Some(lit);
                Ok(())
            } else {
                Ok(())
            }
        });
        table_name
    });

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "MayGetColumn derive requires a #[diesel(table_name = ...)] attribute",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => {
                    return syn::Error::new_spanned(
                        &input,
                        "MayGetColumn can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(&input, "MayGetColumn can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote::quote! {
            impl diesel_additions::MayGetColumn<#table_name::#field_name> for #struct_name {
                fn may_get_column(&self) -> Option<&<#table_name::#field_name as diesel_additions::TypedColumn>::Type> {
                    self.#field_name.as_ref()
                }
            }
        }
    });

    quote::quote! {
        #(#impls)*
    }
    .into()
}

/// Derive macro to automatically implement `SetColumn` and `TrySetColumn` for
/// all fields of a struct.
///
/// This macro generates both `SetColumn` and `TrySetColumn` implementations for
/// each field in the struct, assuming:
/// - The struct has a `#[diesel(table_name = ...)]` attribute
/// - Each field name matches a column name in the table
/// - Each field is wrapped in `Option<T>`
/// - Each column implements `TypedColumn` trait
///
/// The `SetColumn` implementation will set the field to `Some(value.clone())`.
/// The `TrySetColumn` implementation does the same but returns `Ok(())` for
/// compatibility with fallible operations.
#[proc_macro_derive(SetColumn)]
pub fn derive_set_column(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &input.ident;

    // Find the diesel(table_name = ...) attribute
    let table_name = input.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("diesel") {
            return None;
        }

        let mut table_name = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                let value = meta.value()?;
                let lit: syn::Ident = value.parse()?;
                table_name = Some(lit);
                Ok(())
            } else {
                Ok(())
            }
        });
        table_name
    });

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "SetColumn derive requires a #[diesel(table_name = ...)] attribute",
            )
            .to_compile_error()
            .into();
        }
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => {
            match &data.fields {
                syn::Fields::Named(fields) => &fields.named,
                _ => {
                    return syn::Error::new_spanned(
                        &input,
                        "SetColumn can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(&input, "SetColumn can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let impls = fields.iter().flat_map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        vec![
            quote::quote! {
                impl diesel_additions::SetColumn<#table_name::#field_name> for #struct_name {
                    fn set_column(&mut self, value: &<#table_name::#field_name as diesel_additions::TypedColumn>::Type) -> &mut Self {
                        self.#field_name = Some(value.clone());
                        self
                    }
                }
            },
            quote::quote! {
                impl diesel_additions::TrySetColumn<#table_name::#field_name> for #struct_name {
                    fn try_set_column(&mut self, value: &<#table_name::#field_name as diesel_additions::TypedColumn>::Type) -> anyhow::Result<&mut Self> {
                        self.#field_name = Some(value.clone());
                        Ok(self)
                    }
                }
            }
        ]
    });

    quote::quote! {
        #(#impls)*
    }
    .into()
}

/// Derive macro to automatically implement `HasTable` for a struct.
///
/// This macro generates a `HasTable` implementation for the struct,
/// assuming:
/// - The struct has a `#[diesel(table_name = ...)]` attribute
///
/// The implementation provides the associated `Table` type and a `table()`
/// function that returns an instance of the table.
#[proc_macro_derive(HasTable)]
pub fn derive_has_table(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let struct_name = &input.ident;

    // Find the diesel(table_name = ...) attribute
    let table_name = input.attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("diesel") {
            return None;
        }

        let mut table_name = None;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table_name") {
                let value = meta.value()?;
                let lit: syn::Ident = value.parse()?;
                table_name = Some(lit);
                Ok(())
            } else {
                Ok(())
            }
        });
        table_name
    });

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "HasTable derive requires a #[diesel(table_name = ...)] attribute",
            )
            .to_compile_error()
            .into();
        }
    };

    quote::quote! {
        impl diesel::associations::HasTable for #struct_name {
            type Table = #table_name::table;

            fn table() -> Self::Table {
                #table_name::table
            }
        }
    }
    .into()
}

/// Generate `TypedColumn` implementations for all columns in a table
/// definition.
#[proc_macro]
pub fn table_extension(input: TokenStream) -> TokenStream {
    let input_tokens = proc_macro2::TokenStream::from(input.clone());
    let original_input = proc_macro2::TokenStream::from(input);

    // Parse the input to extract table name and columns
    let parsed = match parse_table_definition(input_tokens) {
        Ok(parsed) => parsed,
        Err(err) => return err.to_compile_error().into(),
    };

    let table_name = parsed.table_name;
    let columns = parsed.columns;

    // Generate TypedColumn implementations for each column
    let impls = columns.iter().map(|(col_name, sql_type)| {
        // Normalize the SQL type path
        let normalized_type = normalize_sql_type(sql_type);

        quote::quote! {
            impl diesel_additions::TypedColumn for #table_name::#col_name {
                type Type = <#normalized_type as diesel_additions::RustSqlType>::Type;
            }
        }
    });

    // Generate the complete output including the table! call
    quote::quote! {
        diesel::table! {
            #original_input
        }

        #(#impls)*
    }
    .into()
}

struct TableDefinition {
    table_name: syn::Ident,
    columns: Vec<(syn::Ident, syn::Type)>,
}

fn parse_table_definition(input: proc_macro2::TokenStream) -> syn::Result<TableDefinition> {
    use syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Token,
    };

    struct TableDef {
        table_name: syn::Ident,
        _paren_token: syn::token::Paren,
        _primary_key: syn::Ident,
        _brace_token: syn::token::Brace,
        columns: Punctuated<ColumnDef, Token![,]>,
    }

    struct ColumnDef {
        _attrs: Vec<syn::Attribute>,
        name: syn::Ident,
        _arrow: Token![->],
        sql_type: syn::Type,
    }

    impl Parse for ColumnDef {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let attrs = input.call(syn::Attribute::parse_outer)?;
            let name = input.parse()?;
            let arrow = input.parse()?;
            let sql_type = input.parse()?;
            Ok(ColumnDef { _attrs: attrs, name, _arrow: arrow, sql_type })
        }
    }

    impl Parse for TableDef {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Skip leading attributes
            let _attrs = input.call(syn::Attribute::parse_outer)?;

            let table_name = input.parse()?;
            let content;
            let paren_token = syn::parenthesized!(content in input);
            let primary_key = content.parse()?;

            let content2;
            let brace_token = syn::braced!(content2 in input);
            let columns = content2.parse_terminated(ColumnDef::parse, Token![,])?;

            Ok(TableDef {
                table_name,
                _paren_token: paren_token,
                _primary_key: primary_key,
                _brace_token: brace_token,
                columns,
            })
        }
    }

    let table_def: TableDef = syn::parse2(input)?;

    Ok(TableDefinition {
        table_name: table_def.table_name,
        columns: table_def.columns.into_iter().map(|col| (col.name, col.sql_type)).collect(),
    })
}

fn normalize_sql_type(ty: &syn::Type) -> proc_macro2::TokenStream {
    use quote::ToTokens;

    // Convert the type to a string to analyze it
    let type_str = ty.to_token_stream().to_string();

    // Check if it's already a fully qualified path
    if type_str.starts_with("diesel :: sql_types ::") || type_str.starts_with("diesel::sql_types::")
    {
        return ty.to_token_stream();
    }

    // Otherwise, assume it's a bare type name and prefix it
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let type_name = &type_path.path.segments[0].ident;
            return quote::quote! { diesel::sql_types::#type_name };
        }
    }

    // Fallback: return as-is
    ty.to_token_stream()
}
