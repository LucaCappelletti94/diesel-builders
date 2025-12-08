//! Core tuple generation utilities for procedural macros.
//!
//! This module provides shared functionality for generating trait
//! implementations across tuples of varying sizes (1-32 elements).

use proc_macro2::{Ident, Span, TokenStream};

#[cfg(not(any(
    feature = "size-16",
    feature = "size-32",
    feature = "size-48",
    feature = "size-64",
    feature = "size-96",
    feature = "size-128"
)))]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 8;
#[cfg(all(
    feature = "size-16",
    not(any(
        feature = "size-32",
        feature = "size-48",
        feature = "size-64",
        feature = "size-96",
        feature = "size-128"
    ))
))]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 16;
#[cfg(all(
    feature = "size-32",
    not(any(
        feature = "size-48",
        feature = "size-64",
        feature = "size-96",
        feature = "size-128"
    ))
))]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 32;
#[cfg(all(
    feature = "size-48",
    not(any(feature = "size-64", feature = "size-96", feature = "size-128"))
))]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 48;
#[cfg(all(
    feature = "size-64",
    not(any(feature = "size-96", feature = "size-128"))
))]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 64;
#[cfg(all(feature = "size-96", not(feature = "size-128")))]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 96;
#[cfg(feature = "size-128")]
/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 128;

/// Generate a list of type parameter identifiers (T1, T2, ..., TN)
///
/// # Arguments
///
/// * `count` - The number of type parameters to generate.
pub(crate) fn type_params(count: usize) -> Vec<Ident> {
    type_params_with_prefix(count, "T")
}

pub(crate) fn type_params_with_prefix(count: usize, prefix: &str) -> Vec<Ident> {
    (1..=count)
        .map(|i| Ident::new(&format!("{prefix}{i}"), Span::call_site()))
        .collect()
}

/// Generate all tuple implementations
pub(crate) fn generate_all_sizes<F>(impl_fn: F) -> TokenStream
where
    F: Fn(usize) -> TokenStream,
{
    generate_all_sizes_with_max(MAX_TUPLE_SIZE, impl_fn)
}

/// Generate all tuple implementations from size 1 to `MAX_TUPLE_SIZE`
pub(crate) fn generate_all_sizes_non_empty<F>(impl_fn: F) -> TokenStream
where
    F: Fn(usize) -> TokenStream,
{
    generate_all_sizes_non_empty_with_max(MAX_TUPLE_SIZE, impl_fn)
}

/// Generate all tuple implementations from size 0 (unit) to the specified `max_size`
pub(crate) fn generate_all_sizes_with_max<F>(mut max_size: usize, impl_fn: F) -> TokenStream
where
    F: Fn(usize) -> TokenStream,
{
    max_size = max_size.min(MAX_TUPLE_SIZE);
    (0..=max_size).map(impl_fn).collect()
}

/// Generate all tuple implementations from size 1 to the specified `max_size`
pub(crate) fn generate_all_sizes_non_empty_with_max<F>(max_size: usize, impl_fn: F) -> TokenStream
where
    F: Fn(usize) -> TokenStream,
{
    (1..=max_size).map(impl_fn).collect()
}

mod tests {
    #[test]
    fn test_type_params() {
        let params = super::type_params(3);
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].to_string(), "T1");
        assert_eq!(params[1].to_string(), "T2");
        assert_eq!(params[2].to_string(), "T3");
    }

    #[test]
    fn test_generate_all_sizes() {
        let _generated = super::generate_all_sizes_with_max(8, |size| {
            let struct_ident =
                syn::Ident::new(&format!("TupleSize{size}"), proc_macro2::Span::call_site());
            quote::quote! {
                struct #struct_ident;
            }
        });
    }
}
