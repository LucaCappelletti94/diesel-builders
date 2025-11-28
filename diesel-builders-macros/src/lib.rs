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
