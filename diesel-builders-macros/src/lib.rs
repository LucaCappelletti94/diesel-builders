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
                    fn set_column(&mut self, value: &<#table_name::#field_name as diesel_additions::TypedColumn>::Type) {
                        self.#field_name = Some(value.clone());
                    }
                }
            },
            quote::quote! {
                impl diesel_additions::TrySetColumn<#table_name::#field_name> for #struct_name {
                    fn try_set_column(&mut self, value: &<#table_name::#field_name as diesel_additions::TypedColumn>::Type) -> anyhow::Result<()> {
                        self.#field_name = Some(value.clone());
                        Ok(())
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
