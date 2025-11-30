//! Submodule defining a trait which allows debugging a tuple of any size.

use diesel_builders_macros::impl_debuggable_tuple;

/// A trait for debugging tuples.
///
/// This trait provides a method to create a debug representation of a tuple.
pub trait DebuggableTuple {
    /// Create a debug representation of the tuple.
    fn debug_tuple(&self) -> String;
}

/// Generate implementations for all tuple sizes (0-32)
#[impl_debuggable_tuple]
mod impls {}
