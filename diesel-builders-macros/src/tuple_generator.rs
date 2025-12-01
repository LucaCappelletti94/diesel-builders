//! Core tuple generation utilities for procedural macros.
//!
//! This module provides shared functionality for generating trait
//! implementations across tuples of varying sizes (1-32 elements).

use proc_macro2::{Ident, Span, TokenStream};

/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 32;

/// Generate a list of type parameter identifiers (T1, T2, ..., TN)
///
/// # Arguments
///
/// * `count` - The number of type parameters to generate.
pub(crate) fn type_params(count: usize) -> Vec<Ident> {
    (1..=count)
        .map(|i| Ident::new(&format!("T{i}"), Span::call_site()))
        .collect()
}

/// Generate all tuple implementations from size 0 (unit) to MAX_TUPLE_SIZE
pub(crate) fn generate_all_sizes<F>(impl_fn: F) -> TokenStream
where
    F: Fn(usize) -> TokenStream,
{
    (0..=MAX_TUPLE_SIZE).map(impl_fn).collect()
}
