//! Procedural macros for diesel-builders workspace.
//!
//! This crate provides attribute macros that generate trait implementations
//! for tuples, replacing the complex `macro_rules!` patterns with cleaner
//! procedural macros.

mod impl_generators;
mod tuple_generator;
mod utils;
use proc_macro::TokenStream;

/// Generate `Columns` trait implementations for all tuple sizes.
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

/// Generate `NonEmptyProjection` trait implementations for all tuple sizes.
#[proc_macro_attribute]
pub fn impl_non_empty_projection(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_non_empty_projection();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `Tables`, `TableModels`, and `InsertableTableModels` trait
/// implementations for all tuple sizes.
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
#[proc_macro_attribute]
pub fn impl_get_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_get_columns_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generates tuple trait implementations for `MayGetColumns`.
#[proc_macro_attribute]
pub fn impl_may_get_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_may_get_columns_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generates tuple trait implementations for `SetColumns`.
#[proc_macro_attribute]
pub fn impl_set_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_set_columns_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generates tuple trait implementations for `MaySetColumns`.
#[proc_macro_attribute]
pub fn impl_may_set_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_may_set_columns_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generates tuple trait implementations for `TrySetColumns`.
#[proc_macro_attribute]
pub fn impl_try_set_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_try_set_columns_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generates tuple trait implementations for `TrySetColumnsCollection`.
#[proc_macro_attribute]
pub fn impl_try_set_columns_collection(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_try_set_columns_collections_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generates tuple trait implementations for `TryMaySetColumns`.
#[proc_macro_attribute]
pub fn impl_try_may_set_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_try_may_set_columns_trait();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `BuildableTables` trait implementations for all tuple sizes.
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

/// Generate `BundlableTables` trait implementations for all tuple sizes.
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

/// Generate `AncestorsOf` trait implementations for all tuple sizes.
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
///.
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

/// Generate `TableIndex` trait implementations for all tuple sizes.
#[proc_macro_attribute]
pub fn impl_table_index(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_table_index();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `ForeignKey` trait implementations for all tuple sizes.
#[proc_macro_attribute]
pub fn impl_foreign_key(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_foreign_key();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `TryMaySetDiscretionarySameAsColumns` trait implementations for all tuple sizes.
#[proc_macro_attribute]
pub fn impl_try_may_set_discretionary_same_as_columns(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let impls = impl_generators::generate_try_may_set_discretionary_same_as_columns();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `TrySetMandatorySameAsColumns` trait implementations for all tuple sizes.
#[proc_macro_attribute]
pub fn impl_try_set_mandatory_same_as_columns(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let impls = impl_generators::generate_try_set_mandatory_same_as_columns();
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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "GetColumn derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "GetColumn can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "GetColumn can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote::quote! {
            impl diesel_builders::GetColumn<#table_name::#field_name> for #struct_name {
                fn get_column_ref(&self) -> &<#table_name::#field_name as diesel_builders::Typed>::Type {
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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "MayGetColumn derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "MayGetColumn can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "MayGetColumn can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote::quote! {
            impl diesel_builders::MayGetColumn<#table_name::#field_name> for #struct_name {
                fn may_get_column_ref(&self) -> Option<&<#table_name::#field_name as diesel_builders::Typed>::Type> {
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

/// Derive macro to automatically implement `TrySetColumn` for all fields of a struct.
///
/// This macro generates `TrySetColumn`
/// implementations for each field in the struct, assuming:
/// - The struct has a `#[diesel(table_name = ...)]` attribute
/// - Each field name matches a column name in the table
/// - Each field is wrapped in `Option<T>`
/// - Each column implements `TypedColumn` trait
///
/// The `SetColumn` implementation will set the field to `Some(value.clone())`.
/// The `MaySetColumn` implementation will set the field if the value is `Some`.
/// The `TrySetColumn` implementation does the same as `SetColumn` but returns
/// `Ok(())` for compatibility with fallible operations.
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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "SetColumn derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "SetColumn can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
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
                impl diesel_builders::TrySetColumn<#table_name::#field_name> for #struct_name {
                    type Error = core::convert::Infallible;

                    #[inline]
                    fn try_set_column(&mut self, value: <#table_name::#field_name as diesel_builders::Typed>::Type) -> Result<&mut Self, Self::Error> {
                        self.#field_name = Some(value);
                        Ok(self)
                    }
                }
            },
        ]
    });

    quote::quote! {
        #(#impls)*


            impl diesel_builders::InsertableTableModel for #struct_name
            where
                Self: diesel_builders::HasTableAddition<Table: diesel_builders::TableAddition<InsertableModel = Self>>
            + Default
            + diesel::Insertable<<Self as diesel::associations::HasTable>::Table>
    {
        type Error = core::convert::Infallible;
    }
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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "HasTable derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
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

/// Derive macro to automatically implement `Descendant` trait for root tables.
///
/// This macro should be derived on Model structs to automatically generate
/// the `Descendant` implementation for their associated table type, marking it
/// as a root table with no ancestors.
#[proc_macro_derive(Root)]
pub fn derive_root(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "Root derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
    };

    // Extract struct fields to generate HorizontalSameAsGroup impls
    let fields = match &input.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Root can only be derived on structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Root can only be derived on structs")
                .to_compile_error()
                .into();
        }
    };

    // Generate HorizontalSameAsGroup impl for each field
    let horizontal_impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();

        quote::quote! {
            impl diesel_builders::HorizontalSameAsGroup for #table_name::#field_name {
                type Idx = diesel_builders::typenum::U0;
                type MandatoryHorizontalSameAsKeys = ();
                type DiscretionaryHorizontalSameAsKeys = ();
            }
        }
    });

    quote::quote! {
        impl diesel_builders::Root for #table_name::table {}

        #[diesel_builders_macros::descendant_of]
        impl diesel_builders::Descendant for #table_name::table {
            type Ancestors = ();
            type Root = Self;
        }

        #[diesel_builders_macros::bundlable_table]
        impl BundlableTable for #table_name::table {
            type MandatoryTriangularSameAsColumns = ();
            type DiscretionaryTriangularSameAsColumns = ();
        }

        #(#horizontal_impls)*
    }
    .into()
}

/// Derive macro to automatically implement `TypedColumn` for all table columns.
///
/// This macro should be derived on Model structs to automatically generate
/// `TypedColumn` implementations for each column based on the struct's field
/// types.
#[proc_macro_derive(TableModel)]
#[allow(clippy::too_many_lines)]
pub fn derive_table_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let struct_ident = &input.ident;

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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "TableModel derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "TableModel can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "TableModel can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Extract primary key columns from diesel(primary_key = ...) attribute
    let primary_key_columns: Vec<syn::Ident> = input
        .attrs
        .iter()
        .find_map(|attr| {
            if !attr.path().is_ident("diesel") {
                return None;
            }

            let mut pk_columns = Vec::new();
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("primary_key") {
                    // Parse primary_key(col1, col2, ...)
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let punct: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
                        syn::punctuated::Punctuated::parse_terminated(&content)?;
                    pk_columns.extend(punct);
                    Ok(())
                } else {
                    Ok(())
                }
            });

            if pk_columns.is_empty() {
                None
            } else {
                Some(pk_columns)
            }
        })
        .unwrap_or_else(|| {
            // Default: if no primary_key attribute, assume "id" is the primary key
            vec![syn::Ident::new("id", proc_macro2::Span::call_site())]
        });

    let typed_column_impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        let camel_cased_field_name = utils::snake_to_camel_case(&field_name.to_string());
        let set_field_name =
            syn::Ident::new(&format!("Set{struct_ident}{camel_cased_field_name}"), proc_macro2::Span::call_site());
        let field_name_ref = syn::Ident::new(&format!("{field_name}_ref"), proc_macro2::Span::call_site());
        let try_field_name = syn::Ident::new(&format!("try_{field_name}"), proc_macro2::Span::call_site());
        let try_field_name_ref = syn::Ident::new(&format!("try_{field_name}_ref"), proc_macro2::Span::call_site());
        let try_set_field_name =
            syn::Ident::new(&format!("TrySet{struct_ident}{camel_cased_field_name}"), proc_macro2::Span::call_site());
        let get_field_name =
            syn::Ident::new(&format!("Get{struct_ident}{camel_cased_field_name}"), proc_macro2::Span::call_site());
        let set_field_name_discretionary_model_trait =
            syn::Ident::new(&format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryModel"), proc_macro2::Span::call_site());
        let set_field_name_model_method = syn::Ident::new(&format!("{field_name}_model"), proc_macro2::Span::call_site());
        let set_field_name_model_method_ref = syn::Ident::new(&format!("{field_name}_model_ref"), proc_macro2::Span::call_site());
        let try_set_field_name_model_method = syn::Ident::new(&format!("try_{field_name}_model"), proc_macro2::Span::call_site());
        let try_set_field_name_model_method_ref = syn::Ident::new(&format!("try_{field_name}_model_ref"), proc_macro2::Span::call_site());
        let set_field_name_builder_method = syn::Ident::new(&format!("{field_name}_builder"), proc_macro2::Span::call_site());
        let set_field_name_builder_method_ref = syn::Ident::new(&format!("{field_name}_builder_ref"), proc_macro2::Span::call_site());
        let try_set_field_name_builder_method = syn::Ident::new(&format!("try_{field_name}_builder"), proc_macro2::Span::call_site());
        let try_set_field_name_builder_method_ref = syn::Ident::new(&format!("try_{field_name}_builder_ref"), proc_macro2::Span::call_site());
        let set_field_name_mandatory_builder_trait =
            syn::Ident::new(&format!("Set{struct_ident}{camel_cased_field_name}MandatoryBuilder"), proc_macro2::Span::call_site());
        let set_field_name_discretionary_builder_trait =
            syn::Ident::new(&format!("Set{struct_ident}{camel_cased_field_name}DiscretionaryBuilder"), proc_macro2::Span::call_site());
        let try_set_field_name_discretionary_model_trait =
            syn::Ident::new(&format!("TrySet{struct_ident}{camel_cased_field_name}DiscretionaryModel"), proc_macro2::Span::call_site());
        let try_set_field_name_mandatory_builder_trait =
            syn::Ident::new(&format!("TrySet{struct_ident}{camel_cased_field_name}MandatoryBuilder"), proc_macro2::Span::call_site());
        let try_set_field_name_discretionary_builder_trait =
            syn::Ident::new(&format!("TrySet{struct_ident}{camel_cased_field_name}DiscretionaryBuilder"), proc_macro2::Span::call_site());

        let set_trait_doc_comment = format!(
            "Trait to set the `{field_name}` column on a `{table_name}` table builder."
        );

        let field_name_ref_method_doc_comment = format!(
            "Sets the `{field_name}` column on a `{table_name}` table builder by reference."
        );

        let field_name_method_doc_comment = format!(
            "Sets the `{field_name}` column on a `{table_name}` table builder."
        );

        let try_set_trait_doc_comment = format!(
            "Trait to try to set the `{field_name}` column on a `{table_name}` table builder."
        );

        let try_field_name_ref_method_doc_comment = format!(
            "Tries to set the `{field_name}` column on a `{table_name}` table builder by reference."
        );

        let try_field_name_method_doc_comment = format!(
            "Tries to set the `{field_name}` column on a `{table_name}` table builder."
        );
        let get_trait_doc_comment = format!(
            "Trait to get the `{field_name}` column from a `{table_name}` table model."
        );
        let get_field_name_method_doc_comment = format!(
            "Gets the value of the `{field_name}` column from a `{table_name}` table model."
        );

        let set_discretionary_model_trait_doc_comment = format!(
            "Trait to set the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
        );
        let set_discretionary_model_method_doc_comment = format!(
            "Sets the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
        );
        let set_mandatory_builder_trait_doc_comment = format!(
            "Trait to set the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
        );
        let set_discretionary_builder_trait_doc_comment = format!(
            "Trait to set the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
        );
        let set_mandatory_builder_method_doc_comment = format!(
            "Sets the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
        );
        let set_discretionary_builder_method_doc_comment = format!(
            "Sets the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
        );

        let try_set_discretionary_model_trait_doc_comment = format!(
            "Trait to try to set the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
        );
        let try_set_discretionary_model_method_doc_comment = format!(
            "Tries to set the `{field_name}` column model on a `{table_name}` table builder relative to a discretionary triangular relation."
        );
        let try_set_mandatory_builder_trait_doc_comment = format!(
            "Trait to try to set the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
        );
        let try_set_discretionary_builder_trait_doc_comment = format!(
            "Trait to try to set the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
        );
        let try_set_mandatory_builder_method_doc_comment = format!(
            "Tries to set the `{field_name}` column builder on a `{table_name}` table builder relative to a mandatory triangular relation."
        );
        let try_set_discretionary_builder_method_doc_comment = format!(
            "Tries to set the `{field_name}` column builder on a `{table_name}` table builder relative to a discretionary triangular relation."
        );

        let maybe_getter_impl = (field_name != "id").then(|| {
            quote::quote! {
                #[doc = #get_trait_doc_comment]
                pub trait #get_field_name: diesel_builders::GetColumn<#table_name::#field_name> {
                    #[inline]
                    #[doc = #get_field_name_method_doc_comment]
                    fn #field_name(&self) -> &<#table_name::#field_name as diesel_builders::Typed>::Type {
                        self.get_column_ref()
                    }
                }
                impl<T> #get_field_name for T where T: diesel_builders::GetColumn<#table_name::#field_name> {}
            }
        });

        // The for<'a> bound is needed to satisfy the compiler since the
        // specific `DiscretionarySameAsIndex` is not always implemented
        // for the column type, and it is therefore a trivial bound.
        // See: <https://github.com/rust-lang/rust/issues/48214#issuecomment-2799836786>
        let maybe_triangular_impls = (primary_key_columns.len() == 1).then(|| {
            quote::quote! {
                #[doc = #set_discretionary_model_trait_doc_comment]
                pub trait #set_field_name_discretionary_model_trait: diesel_builders::SetDiscretionaryModel<#table_name::#field_name> + Sized
                    where
                        for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
                {
                    #[inline]
                    #[doc = #set_discretionary_model_method_doc_comment]
                    fn #set_field_name_model_method_ref(
                        &mut self,
                        value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
                    ) -> &mut Self {
                        use diesel_builders::SetDiscretionaryModelExt;
                        self.set_discretionary_model_ref::<#table_name::#field_name>(value)
                    }
                    #[inline]
                    #[must_use]
                    #[doc = #set_discretionary_model_method_doc_comment]
                    fn #set_field_name_model_method(
                        self,
                        value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
                    ) -> Self {
                        use diesel_builders::SetDiscretionaryModelExt;
                        self.set_discretionary_model::<#table_name::#field_name>(value)
                    }
                }

                impl<T> #set_field_name_discretionary_model_trait for T
                    where
                        T: diesel_builders::SetDiscretionaryModel<#table_name::#field_name> + Sized,
                        for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
                    {}

                #[doc = #set_mandatory_builder_trait_doc_comment]
                pub trait #set_field_name_mandatory_builder_trait: diesel_builders::SetMandatoryBuilder<#table_name::#field_name> + Sized
                    where
                        for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
                {
                    #[inline]
                    #[doc = #set_mandatory_builder_method_doc_comment]
                    fn #set_field_name_builder_method_ref(
                        &mut self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> &mut Self {
                        use diesel_builders::SetMandatoryBuilderExt;
                        self.set_mandatory_builder_ref::<#table_name::#field_name>(value)
                    }
                    #[inline]
                    #[must_use]
                    #[doc = #set_mandatory_builder_method_doc_comment]
                    fn #set_field_name_builder_method(
                        self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> Self {
                        use diesel_builders::SetMandatoryBuilderExt;
                        self.set_mandatory_builder::<#table_name::#field_name>(value)
                    }
                }

                impl<T> #set_field_name_mandatory_builder_trait for T
                where
                    T: diesel_builders::SetMandatoryBuilder<#table_name::#field_name> + Sized,
                    for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
                    {}

                #[doc = #set_discretionary_builder_trait_doc_comment]
                pub trait #set_field_name_discretionary_builder_trait: diesel_builders::SetDiscretionaryBuilder<#table_name::#field_name> + Sized
                    where
                        for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
                {
                    #[inline]
                    #[doc = #set_discretionary_builder_method_doc_comment]
                    fn #set_field_name_builder_method_ref(
                        &mut self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> &mut Self {
                        use diesel_builders::SetDiscretionaryBuilderExt;
                        self.set_discretionary_builder_ref::<#table_name::#field_name>(value)
                    }
                    #[inline]
                    #[must_use]
                    #[doc = #set_discretionary_builder_method_doc_comment]
                    fn #set_field_name_builder_method(
                        self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> Self {
                        use diesel_builders::SetDiscretionaryBuilderExt;
                        self.set_discretionary_builder::<#table_name::#field_name>(value)
                    }
                }

                impl<T> #set_field_name_discretionary_builder_trait for T
                where
                    T: diesel_builders::SetDiscretionaryBuilder<#table_name::#field_name> + Sized,
                    for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
                    {}

                #[doc = #try_set_discretionary_model_trait_doc_comment]
                pub trait #try_set_field_name_discretionary_model_trait: diesel_builders::TrySetDiscretionaryModel<#table_name::#field_name> + Sized
                where
                    for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
                {
                    #[inline]
                    #[doc = #try_set_discretionary_model_method_doc_comment]
                    #[doc = ""]
                    #[doc = " # Errors"]
                    #[doc = ""]
                    #[doc = "Returns an error if the column check constraints are not respected."]
                    fn #try_set_field_name_model_method_ref(
                        &mut self,
                        value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
                    ) -> Result<&mut Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                        use diesel_builders::TrySetDiscretionaryModelExt;
                        self.try_set_discretionary_model_ref::<#table_name::#field_name>(value)
                    }
                    #[inline]
                    #[doc = #try_set_discretionary_model_method_doc_comment]
                    #[doc = ""]
                    #[doc = " # Errors"]
                    #[doc = ""]
                    #[doc = "Returns an error if the value cannot be converted to the column type."]
                    fn #try_set_field_name_model_method(
                        self,
                        value: &<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable as diesel_builders::TableAddition>::Model
                    ) -> Result<Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                        use diesel_builders::TrySetDiscretionaryModelExt;
                        self.try_set_discretionary_model::<#table_name::#field_name>(value)
                    }
                }

                impl<T> #try_set_field_name_discretionary_model_trait for T
                where
                    T: diesel_builders::TrySetDiscretionaryModel<#table_name::#field_name> + Sized,
                    for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex
                    {}

                #[doc = #try_set_mandatory_builder_trait_doc_comment]
                pub trait #try_set_field_name_mandatory_builder_trait: diesel_builders::TrySetMandatoryBuilder<#table_name::#field_name> + Sized
                where
                    for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
                {
                    #[inline]
                    #[doc = #try_set_mandatory_builder_method_doc_comment]
                    #[doc = ""]
                    #[doc = " # Errors"]
                    #[doc = ""]
                    #[doc = "Returns an error if the column check constraints are not respected."]
                    fn #try_set_field_name_builder_method_ref(
                        &mut self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> Result<&mut Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                        use diesel_builders::TrySetMandatoryBuilderExt;
                        self.try_set_mandatory_builder_ref::<#table_name::#field_name>(value)
                    }
                    #[inline]
                    #[doc = #try_set_mandatory_builder_method_doc_comment]
                    #[doc = ""]
                    #[doc = " # Errors"]
                    #[doc = ""]
                    #[doc = "Returns an error if the value cannot be converted to the column type."]
                    fn #try_set_field_name_builder_method(
                        self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> Result<Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                        use diesel_builders::TrySetMandatoryBuilderExt;
                        self.try_set_mandatory_builder::<#table_name::#field_name>(value)
                    }
                }

                impl<T> #try_set_field_name_mandatory_builder_trait for T
                where
                    T: diesel_builders::TrySetMandatoryBuilder<#table_name::#field_name> + Sized,
                    for<'a> #table_name::#field_name: diesel_builders::MandatorySameAsIndex<ReferencedTable: BuildableTable>,
                    {}

                #[doc = #try_set_discretionary_builder_trait_doc_comment]
                pub trait #try_set_field_name_discretionary_builder_trait: diesel_builders::TrySetDiscretionaryBuilder<#table_name::#field_name> + Sized
                where
                    for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
                {
                    #[inline]
                    #[doc = #try_set_discretionary_builder_method_doc_comment]
                    #[doc = ""]
                    #[doc = " # Errors"]
                    #[doc = ""]
                    #[doc = "Returns an error if the column check constraints are not respected."]
                    fn #try_set_field_name_builder_method_ref(
                        &mut self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> Result<&mut Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                        use diesel_builders::TrySetDiscretionaryBuilderExt;
                        self.try_set_discretionary_builder_ref::<#table_name::#field_name>(value)
                    }
                    #[inline]
                    #[doc = #try_set_discretionary_builder_method_doc_comment]
                    #[doc = ""]
                    #[doc = " # Errors"]
                    #[doc = ""]
                    #[doc = "Returns an error if the value cannot be converted to the column type."]
                    fn #try_set_field_name_builder_method(
                        self,
                        value: diesel_builders::TableBuilder<<#table_name::#field_name as diesel_builders::SingletonForeignKey>::ReferencedTable>
                    ) -> Result<Self, <<<Self as diesel::associations::HasTable>::Table as diesel_builders::TableAddition>::InsertableModel as diesel_builders::InsertableTableModel>::Error> {
                        use diesel_builders::TrySetDiscretionaryBuilderExt;
                        self.try_set_discretionary_builder::<#table_name::#field_name>(value)
                    }
                }

                impl<T> #try_set_field_name_discretionary_builder_trait for T
                where
                    T: diesel_builders::TrySetDiscretionaryBuilder<#table_name::#field_name> + Sized,
                    for<'a> #table_name::#field_name: diesel_builders::DiscretionarySameAsIndex<ReferencedTable: BuildableTable>,
                    {}
            }
        });

        quote::quote! {
            #maybe_getter_impl
            #maybe_triangular_impls

            #[doc = #set_trait_doc_comment]
            pub trait #set_field_name: diesel_builders::SetColumn<#table_name::#field_name> + Sized {
                #[inline]
                #[doc = #field_name_ref_method_doc_comment]
                fn #field_name_ref(
                    &mut self,
                    value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
                ) -> &mut Self {
                    use diesel_builders::SetColumnExt;
                    self.set_column_ref::<#table_name::#field_name>(value)
                }
                #[inline]
                #[must_use]
                #[doc = #field_name_method_doc_comment]
                fn #field_name(
                    self,
                    value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
                ) -> Self {
                    use diesel_builders::SetColumnExt;
                    self.set_column::<#table_name::#field_name>(value)
                }
            }

            impl<T> #set_field_name for T where T: diesel_builders::SetColumn<#table_name::#field_name> + Sized {}

            #[doc = #try_set_trait_doc_comment]
            pub trait #try_set_field_name: diesel_builders::TrySetColumn<#table_name::#field_name> + Sized {
                #[inline]
                #[doc = #try_field_name_ref_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the column check constraints are not respected."]
                fn #try_field_name_ref(
                    &mut self,
                    value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
                ) -> Result<&mut Self, Self::Error> {
                    use diesel_builders::TrySetColumnExt;
                    self.try_set_column_ref::<#table_name::#field_name>(value)
                }
                #[inline]
                #[doc = #try_field_name_method_doc_comment]
                #[doc = ""]
                #[doc = " # Errors"]
                #[doc = ""]
                #[doc = "Returns an error if the value cannot be converted to the column type."]
                fn #try_field_name(
                    self,
                    value: impl Into<<#table_name::#field_name as diesel_builders::Typed>::Type>
                ) -> Result<Self, Self::Error> {
                    use diesel_builders::TrySetColumnExt;
                    self.try_set_column::<#table_name::#field_name>(value)
                }
            }

            impl<T> #try_set_field_name for T where T: diesel_builders::TrySetColumn<#table_name::#field_name> + Sized {}

            impl diesel_builders::Typed for #table_name::#field_name {
                type Type = #field_type;
            }
        }
    });

    // Generate IndexedColumn implementations for primary key columns
    let pk_column_types: Vec<_> = primary_key_columns
        .iter()
        .map(|col| quote::quote! { #table_name::#col })
        .collect();

    let indexed_column_impls = primary_key_columns.iter().enumerate().map(|(idx, col)| {
        let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
        quote::quote! {
            impl diesel_builders::IndexedColumn<
                diesel_builders::typenum::#idx_type,
                ( #(#pk_column_types,)* )
            > for #table_name::#col {}
        }
    });

    let insertable_ident = syn::Ident::new(
        &format!("New{}", struct_ident),
        proc_macro2::Span::call_site(),
    );

    quote::quote! {
        #(#typed_column_impls)*
        #(#indexed_column_impls)*

        // Auto-implement TableAddition for the table associated with this model.
        impl diesel_builders::TableAddition for #table_name::table {
            type InsertableModel = #insertable_ident;
            type Model = #struct_ident;
            type PrimaryKeyColumns = (#(#table_name::#primary_key_columns,)*) ;
        }
    }
    .into()
}

/// Generate `Descendant` and related trait implementations for a table.
#[proc_macro_attribute]
pub fn descendant_of(attr: TokenStream, item: TokenStream) -> TokenStream {
    match descendant_of_impl(attr, item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[allow(clippy::too_many_lines)]
fn descendant_of_impl(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    use quote::quote;

    let item_impl: syn::ItemImpl = syn::parse(item)?;

    // Extract the table type from the impl block
    let table_type = &item_impl.self_ty;

    // Find the Ancestors associated type
    let mut ancestors_type: Option<&syn::Type> = None;
    let mut root_type: Option<&syn::Type> = None;

    for item in &item_impl.items {
        if let syn::ImplItem::Type(type_item) = item {
            if type_item.ident == "Ancestors" {
                ancestors_type = Some(&type_item.ty);
            } else if type_item.ident == "Root" {
                root_type = Some(&type_item.ty);
            }
        }
    }

    let ancestors_type = ancestors_type
        .ok_or_else(|| syn::Error::new_spanned(&item_impl, "Missing Ancestors associated type"))?;

    let root_type = root_type
        .ok_or_else(|| syn::Error::new_spanned(&item_impl, "Missing Root associated type"))?;

    // Parse the ancestors from the type - it should be a tuple like (T1, T2, T3) or
    // ()
    let ancestors = extract_tuple_types(ancestors_type)?;

    let num_ancestors = ancestors.len();

    // Generate TupleIndex for self (last position in ancestors + self)
    let self_idx = syn::Ident::new(&format!("U{num_ancestors}"), proc_macro2::Span::call_site());

    // Generate DescendantOf implementations for each direct ancestor
    let descendant_of_impls: Vec<_> = ancestors
        .iter()
        .map(|ancestor| {
            quote! {
                impl diesel_builders::DescendantOf<#ancestor> for #table_type {}
            }
        })
        .collect();

    // Generate `GetColumn` implementation for each ancestor's primary key
    // for the descendant table model
    let descendant_table_model = quote! {
        <#table_type as diesel_builders::TableAddition>::Model
    };
    let get_column_impls : Vec<_> = ancestors
        .iter()
        .map(|ancestor| {
            quote! {
                impl diesel_builders::GetColumn<<#ancestor as diesel::Table>::PrimaryKey>
                    for #descendant_table_model
                {
                    fn get_column_ref(
                        &self,
                    ) -> &<<#ancestor as diesel::Table>::PrimaryKey as diesel_builders::Typed>::Type {
                        use diesel::Identifiable;
                        self.id()
                    }
                }
            }
        })
        .collect();

    // Generate DescendantOf implementation for the root (if it's not already in
    // ancestors) We need to check if root_type is different from all ancestors
    let root_descendant_of_impl = if ancestors.is_empty() {
        quote! {}
    } else {
        // Check if the root is already in the ancestors list by comparing token streams
        let root_tokens = quote! { #root_type }.to_string();
        let is_root_in_ancestors = ancestors.iter().any(|ancestor| {
            let ancestor_tokens = quote! { #ancestor }.to_string();
            ancestor_tokens == root_tokens
        });

        if is_root_in_ancestors {
            quote! {}
        } else {
            quote! {
                impl diesel_builders::DescendantOf<#root_type> for #table_type {}
            }
        }
    };

    // Generate AncestorOfIndex implementations for each ancestor
    let ancestor_of_index_impls: Vec<_> = ancestors
        .iter()
        .enumerate()
        .map(|(i, ancestor)| {
            let idx = syn::Ident::new(&format!("U{i}"), proc_macro2::Span::call_site());
            quote! {
                impl diesel_builders::AncestorOfIndex<#table_type> for #ancestor {
                    type Idx = diesel_builders::typenum::#idx;
                }
            }
        })
        .collect();

    // Generate AncestorOfIndex for self
    let self_ancestor_of_index = quote! {
        impl diesel_builders::AncestorOfIndex<#table_type> for #table_type {
            type Idx = diesel_builders::typenum::#self_idx;
        }
    };

    Ok(quote! {
        #item_impl

        #(#descendant_of_impls)*

        #root_descendant_of_impl

        #self_ancestor_of_index

        #(#get_column_impls)*

        #(#ancestor_of_index_impls)*
    }
    .into())
}

/// Helper function to extract types from a tuple type
fn extract_tuple_types(ty: &syn::Type) -> syn::Result<Vec<syn::Type>> {
    match ty {
        syn::Type::Tuple(tuple) => Ok(tuple.elems.iter().cloned().collect()),
        _ => Err(syn::Error::new_spanned(
            ty,
            "Expected a tuple type for Ancestors",
        )),
    }
}

/// Generate `MandatorySameAsIndex` and `DiscretionarySameAsIndex` trait
/// implementations for a table.
///
/// This macro should be applied to the `impl BundlableTable for table` block.
/// It will automatically generate index trait implementations for each column
/// listed in `MandatoryTriangularSameAsColumns` and
/// `DiscretionaryTriangularSameAsColumns`.
#[proc_macro_attribute]
pub fn bundlable_table(attr: TokenStream, item: TokenStream) -> TokenStream {
    match bundlable_table_impl(attr, item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

fn bundlable_table_impl(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    use quote::quote;

    let item_impl: syn::ItemImpl = syn::parse(item)?;

    // Find the MandatoryTriangularSameAsColumns and
    // DiscretionaryTriangularSameAsColumns associated types
    let mut mandatory_columns_type: Option<&syn::Type> = None;
    let mut discretionary_columns_type: Option<&syn::Type> = None;

    for item in &item_impl.items {
        if let syn::ImplItem::Type(type_item) = item {
            if type_item.ident == "MandatoryTriangularSameAsColumns" {
                mandatory_columns_type = Some(&type_item.ty);
            } else if type_item.ident == "DiscretionaryTriangularSameAsColumns" {
                discretionary_columns_type = Some(&type_item.ty);
            }
        }
    }

    let mandatory_columns_type = mandatory_columns_type.ok_or_else(|| {
        syn::Error::new_spanned(
            &item_impl,
            "Missing MandatoryTriangularSameAsColumns associated type",
        )
    })?;

    let discretionary_columns_type = discretionary_columns_type.ok_or_else(|| {
        syn::Error::new_spanned(
            &item_impl,
            "Missing DiscretionaryTriangularSameAsColumns associated type",
        )
    })?;

    // Parse the columns from the types - they should be tuples like (C1, C2, C3) or
    // ()
    let mandatory_columns = extract_tuple_types(mandatory_columns_type)?;
    let discretionary_columns = extract_tuple_types(discretionary_columns_type)?;

    // Generate MandatorySameAsIndex implementations for each mandatory column
    let mandatory_impls: Vec<_> = mandatory_columns
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

    // Generate DiscretionarySameAsIndex implementations for each discretionary
    // column
    let discretionary_impls: Vec<_> = discretionary_columns
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

    let table_type = &item_impl.self_ty;

    let primary_key_group = if !mandatory_columns.is_empty() || !discretionary_columns.is_empty() {
        Some(
            quote! {impl diesel_builders::HorizontalSameAsGroup for <#table_type as diesel::Table>::PrimaryKey {
                type Idx = diesel_builders::typenum::U0;
                type MandatoryHorizontalSameAsKeys = (#(#mandatory_columns,)*);
                type DiscretionaryHorizontalSameAsKeys = (#(#discretionary_columns,)*);
            }},
        )
    } else {
        None
    };

    Ok(quote! {
        #item_impl

        #primary_key_group

        #(#mandatory_impls)*

        #(#discretionary_impls)*
    }
    .into())
}

/// Derive macro to automatically implement `HorizontalSameAsGroup` for all
/// columns in a model struct with empty tuples.
///
/// This macro generates `HorizontalSameAsGroup` implementations for each column
/// in the struct, setting both `MandatoryHorizontalSameAsKeys` and
/// `DiscretionaryHorizontalSameAsKeys` to `()`.
#[proc_macro_derive(Decoupled)]
pub fn derive_no_horizontal_same_as_group(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

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

    let Some(table_name) = table_name else {
        return syn::Error::new_spanned(
            &input,
            "NoHorizontalSameAsGroup derive requires a #[diesel(table_name = ...)] attribute",
        )
        .to_compile_error()
        .into();
    };

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "NoHorizontalSameAsGroup can only be derived for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "NoHorizontalSameAsGroup can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote::quote! {
            impl diesel_builders::HorizontalSameAsGroup for #table_name::#field_name {
                type Idx = diesel_builders::typenum::U0;
                type MandatoryHorizontalSameAsKeys = ();
                type DiscretionaryHorizontalSameAsKeys = ();
            }
        }
    });

    quote::quote! {
        impl BundlableTable for #table_name::table {
            type MandatoryTriangularSameAsColumns = ();
            type DiscretionaryTriangularSameAsColumns = ();
        }

        #(#impls)*
    }
    .into()
}

/// Define a foreign key relationship using SQL-like syntax.
///
/// This macro generates `HostColumn` implementations for each column in the foreign key.
/// The `ForeignKey` trait implementation is automatically provided by the `#[impl_foreign_key]`
/// procedural macro when all columns implement `HostColumn`.
#[proc_macro]
pub fn fk(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Token, Type,
    };

    struct ForeignKey {
        host_columns: Punctuated<Type, Token![,]>,
        ref_columns: Punctuated<Type, Token![,]>,
    }

    impl Parse for ForeignKey {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Parse: ( host_cols ) REFERENCES ( ref_cols )
            let host_content;
            syn::parenthesized!(host_content in input);
            let host_columns = Punctuated::parse_terminated(&host_content)?;

            input.parse::<syn::Ident>()?; // REFERENCES keyword

            let ref_content;
            syn::parenthesized!(ref_content in input);
            let ref_columns = Punctuated::parse_terminated(&ref_content)?;

            Ok(ForeignKey {
                host_columns,
                ref_columns,
            })
        }
    }

    let fk_def = syn::parse_macro_input!(input as ForeignKey);
    let host_cols: Vec<_> = fk_def.host_columns.iter().collect();
    let ref_cols: Vec<_> = fk_def.ref_columns.iter().collect();

    if host_cols.len() != ref_cols.len() {
        return syn::Error::new_spanned(
            &fk_def.host_columns,
            "Number of host columns must match number of referenced columns",
        )
        .to_compile_error()
        .into();
    }

    // Generate HostColumn implementation for each column at its index
    let impls = host_cols.iter().enumerate().map(|(idx, host_col)| {
        let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
        quote! {
            impl diesel_builders::HostColumn<
                diesel_builders::typenum::#idx_type,
                ( #(#host_cols,)* ),
                ( #(#ref_cols,)* )
            > for #host_col {}
        }
    });

    quote! {
        #(#impls)*
    }
    .into()
}

/// Define a table index using SQL-like syntax.
///
/// This macro generates `IndexedColumn` implementations for each column in the index.
/// The `TableIndex` trait implementation is automatically provided by the `#[impl_table_index]`
/// procedural macro when all columns implement `IndexedColumn`.
#[proc_macro]
pub fn index(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        Token, Type,
    };

    struct TableIndex {
        columns: Punctuated<Type, Token![,]>,
    }

    impl Parse for TableIndex {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Parse: ( cols )
            let content;
            syn::parenthesized!(content in input);
            let columns = Punctuated::parse_terminated(&content)?;

            Ok(TableIndex { columns })
        }
    }

    let index_def = syn::parse_macro_input!(input as TableIndex);
    let cols: Vec<_> = index_def.columns.iter().collect();

    // Generate IndexedColumn implementation for each column at its index
    let impls = cols.iter().enumerate().map(|(idx, col)| {
        let idx_type = syn::Ident::new(&format!("U{idx}"), proc_macro2::Span::call_site());
        quote! {
            impl diesel_builders::IndexedColumn<
                diesel_builders::typenum::#idx_type,
                ( #(#cols,)* )
            > for #col {}
        }
    });

    quote! {
        #(#impls)*
    }
    .into()
}
