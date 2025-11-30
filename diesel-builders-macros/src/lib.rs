//! Procedural macros for diesel-builders workspace.
//!
//! This crate provides attribute macros that generate trait implementations
//! for tuples, replacing the complex `macro_rules!` patterns with cleaner
//! procedural macros.

mod impl_generators;
mod tuple_generator;

use proc_macro::TokenStream;

/// Generate `DefaultTuple` trait implementations for all tuple sizes (1-32).
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

/// Generate `CompletedTableBuilder` `NestedInsert` trait implementations for
/// all tuple sizes (2-32).
///
/// This generates the recursive nested insert implementations for
/// `CompletedTableBuilder` with varying tuple sizes. The size 1 case is handled
/// separately as a base case.
#[proc_macro_attribute]
pub fn impl_completed_table_builder_nested_insert(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let impls = impl_generators::generate_completed_table_builder_nested_insert();
    let item = proc_macro2::TokenStream::from(item);

    quote::quote! {
        #item
        #impls
    }
    .into()
}

/// Generate `BuildableTables` trait implementations for all tuple sizes (1-32).
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

/// Generate `TrySetMandatorySameAsColumns` and
/// `TrySetDiscretionarySameAsColumns` trait implementations for all tuple sizes
/// (0-32).
#[proc_macro_attribute]
pub fn impl_try_set_same_as_columns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impls = impl_generators::generate_try_set_same_as_columns();
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

/// Derive macro to automatically implement `SetColumn`, `MaySetColumn`, and
/// `TrySetColumn` for all fields of a struct.
///
/// This macro generates `SetColumn`, `MaySetColumn`, and `TrySetColumn`
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
                    #[inline]
                    fn set_column(&mut self, value: &<#table_name::#field_name as diesel_additions::TypedColumn>::Type) -> &mut Self {
                        self.#field_name = Some(value.clone());
                        self
                    }
                }
            },
            quote::quote! {
                impl diesel_additions::MaySetColumn<#table_name::#field_name> for #struct_name {
                    #[inline]
                    fn may_set_column(&mut self, value: Option<&<#table_name::#field_name as diesel_additions::TypedColumn>::Type>) -> &mut Self {
                        if let Some(v) = value {
                            self.#field_name = Some(v.clone());
                        }
                        self
                    }
                }
            },
            quote::quote! {
                impl diesel_additions::TrySetColumn<#table_name::#field_name> for #struct_name {
                    #[inline]
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

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "Root derive requires a #[diesel(table_name = ...)] attribute",
            )
            .to_compile_error()
            .into();
        }
    };

    quote::quote! {
        impl diesel_relations::Root for #table_name::table {}

        #[diesel_builders_macros::descendant_of]
        impl diesel_relations::Descendant for #table_name::table {
            type Ancestors = ();
            type Root = Self;
        }

        #[diesel_builders_macros::bundlable_table]
        impl BundlableTable for #table_name::table {
            type MandatoryTriangularSameAsColumns = ();
            type DiscretionaryTriangularSameAsColumns = ();
        }
    }
    .into()
}

/// Derive macro to automatically implement `TypedColumn` for all table columns.
///
/// This macro should be derived on Model structs to automatically generate
/// `TypedColumn` implementations for each column based on the struct's field
/// types.
#[proc_macro_derive(TableModel)]
pub fn derive_table_model(input: TokenStream) -> TokenStream {
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

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "TableModel derive requires a #[diesel(table_name = ...)] attribute",
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
                        "TableModel can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(&input, "TableModel can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    let impls = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        quote::quote! {
            impl diesel_additions::TypedColumn for #table_name::#field_name {
                type Type = #field_type;
            }
        }
    });

    quote::quote! {
        #(#impls)*
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
    let self_idx =
        syn::Ident::new(&format!("TupleIndex{}", num_ancestors), proc_macro2::Span::call_site());

    // Generate DescendantOf implementations for each direct ancestor
    let descendant_of_impls: Vec<_> = ancestors
        .iter()
        .map(|ancestor| {
            quote! {
                impl diesel_relations::DescendantOf<#ancestor> for #table_type {}
            }
        })
        .collect();

    // Generate DescendantOf implementation for the root (if it's not already in
    // ancestors) We need to check if root_type is different from all ancestors
    let root_descendant_of_impl = if !ancestors.is_empty() {
        // Check if the root is already in the ancestors list by comparing token streams
        let root_tokens = quote! { #root_type }.to_string();
        let is_root_in_ancestors = ancestors.iter().any(|ancestor| {
            let ancestor_tokens = quote! { #ancestor }.to_string();
            ancestor_tokens == root_tokens
        });

        if !is_root_in_ancestors {
            quote! {
                impl diesel_relations::DescendantOf<#root_type> for #table_type {}
            }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    // Generate AncestorOfIndex implementations for each ancestor
    let ancestor_of_index_impls: Vec<_> = ancestors
        .iter()
        .enumerate()
        .map(|(i, ancestor)| {
            let idx = syn::Ident::new(&format!("TupleIndex{}", i), proc_macro2::Span::call_site());
            quote! {
                impl diesel_relations::AncestorOfIndex<#table_type> for #ancestor {
                    type Idx = typed_tuple::prelude::#idx;
                }
            }
        })
        .collect();

    // Generate AncestorOfIndex for self
    let self_ancestor_of_index = quote! {
        impl diesel_relations::AncestorOfIndex<#table_type> for #table_type {
            type Idx = typed_tuple::prelude::#self_idx;
        }
    };

    // Generate VerticalSameAsGroup implementations for all ancestors
    let vertical_same_as_impls: Vec<_> = if !ancestors.is_empty() {
        let non_root_ancestors = &ancestors[1..];
        ancestors
            .iter()
            .enumerate()
            .map(|(i, ancestor)| {
                if i == 0 {
                    // For the root, use the descendant table's primary key
                    quote! {
                        impl diesel_relations::vertical_same_as_group::VerticalSameAsGroup<#table_type>
                            for <#ancestor as diesel::Table>::PrimaryKey
                        {
                            type VerticalSameAsColumns = (
                                #(<#non_root_ancestors as diesel::Table>::PrimaryKey,)*
                                <#table_type as diesel::Table>::PrimaryKey,
                            );
                        }
                    }
                } else {
                    // For intermediate ancestors, use an empty tuple
                    quote! {
                        impl diesel_relations::vertical_same_as_group::VerticalSameAsGroup<#table_type>
                            for <#ancestor as diesel::Table>::PrimaryKey
                        {
                            type VerticalSameAsColumns = ();
                        }
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(quote! {
        #item_impl

        #(#descendant_of_impls)*

        #root_descendant_of_impl

        #self_ancestor_of_index

        #(#ancestor_of_index_impls)*

        #(#vertical_same_as_impls)*
    }
    .into())
}

/// Helper function to extract types from a tuple type
fn extract_tuple_types(ty: &syn::Type) -> syn::Result<Vec<syn::Type>> {
    match ty {
        syn::Type::Tuple(tuple) => Ok(tuple.elems.iter().cloned().collect()),
        _ => Err(syn::Error::new_spanned(ty, "Expected a tuple type for Ancestors")),
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
            let idx = syn::Ident::new(&format!("TupleIndex{}", i), proc_macro2::Span::call_site());
            quote! {
                impl diesel_relations::MandatorySameAsIndex for #column {
                    type Idx = typed_tuple::prelude::#idx;
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
            let idx = syn::Ident::new(&format!("TupleIndex{}", i), proc_macro2::Span::call_site());
            quote! {
                impl diesel_relations::DiscretionarySameAsIndex for #column {
                    type Idx = typed_tuple::prelude::#idx;
                }
            }
        })
        .collect();

    let table_type = &item_impl.self_ty;

    let primary_key_group = if !mandatory_columns.is_empty() || !discretionary_columns.is_empty() {
        Some(
            quote! {impl diesel_relations::HorizontalSameAsGroup for <#table_type as diesel::Table>::PrimaryKey {
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
#[proc_macro_derive(NoHorizontalSameAsGroup)]
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

    let table_name = match table_name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(
                &input,
                "NoHorizontalSameAsGroup derive requires a #[diesel(table_name = ...)] attribute",
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
                        "NoHorizontalSameAsGroup can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
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
            impl diesel_relations::HorizontalSameAsGroup for #table_name::#field_name {
                type MandatoryHorizontalSameAsKeys = ();
                type DiscretionaryHorizontalSameAsKeys = ();
            }
        }
    });

    quote::quote! {
        #(#impls)*
    }
    .into()
}
