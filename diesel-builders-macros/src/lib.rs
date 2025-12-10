//! Procedural macros for diesel-builders workspace.
//!
//! This crate provides attribute macros that generate trait implementations
//! for tuples, replacing the complex `macro_rules!` patterns with cleaner
//! procedural macros.

mod table_model;
mod utils;
use proc_macro::TokenStream;

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

    let impls = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;
        Some(quote::quote! {
            impl diesel_builders::MayGetColumn<#table_name::#field_name> for #struct_name {
                fn may_get_column_ref<'a>(&'a self) -> Option<&'a <#table_name::#field_name as diesel_builders::Typed>::Type>
                    where
                        #table_name::table: 'a,
                    {
                    self.#field_name.as_ref()
                }
            }
        })
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

    let impls = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;
        Some(
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
        )
    });

    quote::quote! {
        #(#impls)*


            impl diesel_builders::InsertableTableModel for #struct_name
            where
                Self: diesel_builders::HasTableExt<Table: diesel_builders::TableExt<InsertableModel = Self>>
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
    let horizontal_impls = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;

        Some(quote::quote! {
            impl diesel_builders::HorizontalSameAsGroup for #table_name::#field_name {
                type Idx = diesel_builders::typenum::U0;
                type MandatoryHorizontalKeys = ();
                type DiscretionaryHorizontalKeys = ();
            }
        })
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
            type MandatoryTriangularColumns = ();
            type DiscretionaryTriangularColumns = ();
        }

        #(#horizontal_impls)*
    }
    .into()
}

/// Derive macro to automatically implement `TypedColumn` for all table columns.
///
/// This macro should be derived on Model structs to automatically generate
/// `TypedColumn` implementations for each column based on the struct's field
/// types. It also automatically implements `GetColumn` for all fields, replacing
/// the need for a separate `GetColumn` derive.
///
/// Supports a helper attribute to override the insertable model name:
/// ```ignore
/// #[derive(TableModel)]
/// #[table_model(insertable = MyCustomInsertable)]
/// #[diesel(table_name = my_table)]
/// struct MyModel { ... }
/// ```
#[proc_macro_derive(TableModel, attributes(table_model))]
pub fn derive_table_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match table_model::derive_table_model_impl(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
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
/// Implementation for the `#[descendant_of]` attribute macro.
///
/// This function parses a `impl` block for a table type and generates
/// the `Descendant`, `DescendantOf`, `AncestorOfIndex` and `GetColumn`
/// implementations required by the `diesel-builders` runtime. It inspects the
/// `Ancestors` and `Root` associated types defined in the `impl` block, and
/// emits the boilerplate implementations that allow composition of nested
/// builder operations.
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
        <#table_type as diesel_builders::TableExt>::Model
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
/// listed in `MandatoryTriangularColumns` and
/// `DiscretionaryTriangularColumns`.
#[proc_macro_attribute]
pub fn bundlable_table(attr: TokenStream, item: TokenStream) -> TokenStream {
    match bundlable_table_impl(attr, item) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implementation for the `#[bundlable_table]` attribute macro.
///
/// This function parses a `impl BundlableTable for <table>` block, extracts the
/// `MandatoryTriangularColumns` and `DiscretionaryTriangularColumns` associated
/// types, and generates the `MandatorySameAsIndex` and `DiscretionarySameAsIndex`
/// trait implementations for each column referenced. When the table has
/// non-empty triangular columns, a `HorizontalSameAsGroup` impl is generated
/// for the primary key to group them together.
fn bundlable_table_impl(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    use quote::quote;

    let item_impl: syn::ItemImpl = syn::parse(item)?;

    // Find the MandatoryTriangularColumns and
    // DiscretionaryTriangularColumns associated types
    let mut mandatory_columns_type: Option<&syn::Type> = None;
    let mut discretionary_columns_type: Option<&syn::Type> = None;

    for item in &item_impl.items {
        if let syn::ImplItem::Type(type_item) = item {
            if type_item.ident == "MandatoryTriangularColumns" {
                mandatory_columns_type = Some(&type_item.ty);
            } else if type_item.ident == "DiscretionaryTriangularColumns" {
                discretionary_columns_type = Some(&type_item.ty);
            }
        }
    }

    let mandatory_columns_type = mandatory_columns_type.ok_or_else(|| {
        syn::Error::new_spanned(
            &item_impl,
            "Missing MandatoryTriangularColumns associated type",
        )
    })?;

    let discretionary_columns_type = discretionary_columns_type.ok_or_else(|| {
        syn::Error::new_spanned(
            &item_impl,
            "Missing DiscretionaryTriangularColumns associated type",
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
                type MandatoryHorizontalKeys = (#(#mandatory_columns,)*);
                type DiscretionaryHorizontalKeys = (#(#discretionary_columns,)*);
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
/// in the struct, setting both `MandatoryHorizontalKeys` and
/// `DiscretionaryHorizontalKeys` to `()`.
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

    let impls = fields.iter().filter_map(|field| {
        let field_name = field.ident.as_ref()?;
        Some(quote::quote! {
            impl diesel_builders::HorizontalSameAsGroup for #table_name::#field_name {
                type Idx = diesel_builders::typenum::U0;
                type MandatoryHorizontalKeys = ();
                type DiscretionaryHorizontalKeys = ();
            }
        })
    });

    quote::quote! {
        impl BundlableTable for #table_name::table {
            type MandatoryTriangularColumns = ();
            type DiscretionaryTriangularColumns = ();
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

    /// Parsed representation of a `FOREIGN KEY` specification provided to
    /// the `fk!()` macro. `host_columns` are the columns on the local table,
    /// while `ref_columns` are the corresponding columns on the foreign table.
    struct ForeignKey {
        /// The host/source columns that form the foreign key on the local table.
        host_columns: Punctuated<Type, Token![,]>,
        /// The referenced/target columns that the host columns point to.
        ref_columns: Punctuated<Type, Token![,]>,
    }

    impl Parse for ForeignKey {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Parse: ( host_cols ) -> ( ref_cols )
            let host_content;
            syn::parenthesized!(host_content in input);
            let host_columns = Punctuated::parse_terminated(&host_content)?;

            input.parse::<Token![->]>()?;

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

    /// Parsed representation of an `INDEX` macro invocation, containing the
    /// columns that form the index definition.
    struct TableIndex {
        /// The columns included in the index.
        columns: Punctuated<Type, Token![,]>,
    }

    impl Parse for TableIndex {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            // Accept either: a parenthesized tuple `(col1, col2)` or a bare list `col1, col2`.
            if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let columns = Punctuated::parse_terminated(&content)?;
                Ok(TableIndex { columns })
            } else {
                let columns = Punctuated::parse_terminated(input)?;
                Ok(TableIndex { columns })
            }
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

/// Macro to declare a singleton foreign key relationship.
///
/// This macro:
/// 1. Implements `ForeignPrimaryKey` for a column that references another table's primary key
/// 2. Generates a helper trait with a method to fetch the foreign record using `GetForeignExt`
///
/// # Method naming
/// - If column name ends with `_id` (e.g., `a_id`), the method will be named `a`
/// - Otherwise, the method will be named `{column_name}_fk`
///
/// # Example
/// ```ignore
/// fpk!(table_b::c_id -> table_c);
/// ```
///
/// This generates:
/// - `impl ForeignPrimaryKey for table_b::c_id { type ReferencedTable = table_c::table; }`
/// - A trait `GetTableBC` with method `c(&self, conn: &mut Conn)` that calls `get_foreign`
#[proc_macro]
#[allow(clippy::too_many_lines)]
pub fn fpk(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::{
        parse::{Parse, ParseStream},
        Path, Token,
    };

    /// Parsed representation of a singleton foreign key declaration.
    struct ForeignPrimaryKeyDecl {
        /// The column that is the foreign key (e.g., `table_b::c_id`)
        column: Path,
        /// The referenced table (e.g., `table_c`)
        referenced_table: Path,
    }

    impl Parse for ForeignPrimaryKeyDecl {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let column: Path = input.parse()?;
            input.parse::<Token![->]>()?;
            let referenced_table: Path = input.parse()?;
            Ok(ForeignPrimaryKeyDecl {
                column,
                referenced_table,
            })
        }
    }

    let decl = syn::parse_macro_input!(input as ForeignPrimaryKeyDecl);
    let column = decl.column;
    let referenced_table = decl.referenced_table;

    // Extract column name for method generation — must be present
    let column_name = match column.segments.last() {
        Some(seg) => seg.ident.to_string(),
        None => {
            return syn::Error::new_spanned(
                &column,
                "fpk! macro requires a column path (e.g., `table_b::c_id`). Could not extract column name.",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract referenced table name for method generation — must be present
    let referenced_table_name = match referenced_table.segments.last() {
        Some(seg) => seg.ident.to_string(),
        None => {
            // If we cannot extract a referenced table name, fail the macro with a helpful message
            return syn::Error::new_spanned(
                &referenced_table,
                "fpk! macro requires a referenced table path (e.g., `table_c`). Could not extract referenced table name.",
            )
            .to_compile_error()
            .into();
        }
    };

    // Generate method name based on column name
    let method_name = if let Some(stripped) = column_name.strip_suffix("_id") {
        stripped.to_string()
    } else {
        format!("{column_name}_fk")
    };
    let method_ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

    // Generate trait name
    // Extract table name from column path (second-to-last segment)
    let table_name_segment = if column.segments.len() >= 2 {
        column.segments[column.segments.len() - 2].ident.to_string()
    } else {
        "table".to_string()
    };

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

    quote! {
        impl diesel_builders::ForeignPrimaryKey for #column {
            type ReferencedTable = #referenced_table::table;
        }

        #[doc = #trait_doc]
        pub trait #trait_ident<Conn>: diesel_builders::GetForeignExt<Conn> {
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
            ) -> diesel::QueryResult<<#referenced_table::table as diesel_builders::TableExt>::Model>
            where
                Self: diesel_builders::GetForeign<
                    Conn,
                    (#column,),
                    (<#referenced_table::table as diesel::Table>::PrimaryKey,),
                >,
            {
                <Self as diesel_builders::GetForeign<
                    Conn,
                    (#column,),
                    (<#referenced_table::table as diesel::Table>::PrimaryKey,),
                >>::get_foreign(self, conn)
            }
        }

        impl<T, Conn> #trait_ident<Conn> for T
        where
            T: diesel_builders::GetForeign<
                Conn,
                (#column,),
                (<#referenced_table::table as diesel::Table>::PrimaryKey,)
            > {}
    }
    .into()
}
