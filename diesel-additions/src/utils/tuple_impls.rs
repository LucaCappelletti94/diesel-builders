//! Common macro for generating tuple implementations.
//!
//! This module provides a shared macro to ensure all tuple implementations
//! use the same maximum size and maintain consistency across the codebase.

/// Maximum number of elements supported in tuple implementations.
pub const MAX_TUPLE_SIZE: usize = 32;

/// Generate tuple implementations for a recursive macro.
///
/// This macro invokes the provided macro name with a standardized list of
/// type parameters representing tuples up to 32 elements. This ensures
/// consistency across all tuple-based trait implementations in the codebase.
#[macro_export]
macro_rules! generate_tuple_impls {
    ($macro_name:ident) => {
        $macro_name!(
            T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
            T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
        );
    };
}
