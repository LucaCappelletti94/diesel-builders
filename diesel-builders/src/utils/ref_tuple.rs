//! Submodule defining a trait which, for any tuple (T1, T2, ...), defines
//! an associated type which is the `(&'a T1, &'a T2, ...)` tuple.

use diesel_builders_macros::impl_ref_tuple;

use crate::OptionTuple;

/// A trait for defining a tuple type's corresponding reference tuple type.
pub trait RefTuple {
    /// The associated reference tuple type.
    type Output<'a>: OptionTuple
    where
        Self: 'a;
}

/// Generate implementations for all tuple sizes (1-32)
#[impl_ref_tuple]
mod impls {}
