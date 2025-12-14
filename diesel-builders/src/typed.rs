//! Submodule providing the `Typed` trait.

mod typed_nested_tuple;
pub use typed_nested_tuple::*;
mod typed_tuple;
pub use typed_tuple::*;
mod typed_nested_tuple_collection;
pub use typed_nested_tuple_collection::*;
mod homogeneously_typed_nested_tuple;
pub use homogeneously_typed_nested_tuple::*;
mod homogeneously_typed_tuple;
pub use homogeneously_typed_tuple::*;

/// Trait representing an object with an associated type.
pub trait Typed {
    /// The value type associated with this object, as it appears in queries.
    type ValueType: Clone;
    /// The column type associated with this object, which may be an `Option` of the value type.
    type ColumnType: Clone;
}
