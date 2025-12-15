//! Submodule providing the `Typed` trait.

mod typed_nested_tuple;
use std::fmt::Debug;

pub use typed_nested_tuple::*;
mod typed_nested_tuple_collection;
pub use typed_nested_tuple_collection::*;
mod homogeneously_typed_nested_tuple;
pub use homogeneously_typed_nested_tuple::*;

/// Trait representing an object with an associated type.
pub trait Typed {
    /// The value type associated with this object, as it appears in queries.
    type ValueType: Clone;
    /// The column type associated with this object, which may be an `Option` of the value type.
    type ColumnType: Clone + Debug + From<Self::ValueType> + Into<Option<Self::ValueType>>;
}

/// Trait representing an object whose `ValueType` and `ColumnType` are the same,
/// and therefore cannot be optional.
pub trait NonOptionalTyped: Typed<ValueType = <Self as Typed>::ColumnType> {}

impl<T> NonOptionalTyped for T where T: Typed<ValueType = <T as Typed>::ColumnType> {}

/// Trait representing an object whose `ColumnType` is an `Option` of its `ValueType`.
pub trait OptionalTyped: Typed<ColumnType = Option<<Self as Typed>::ValueType>> {}

impl<T> OptionalTyped for T where T: Typed<ColumnType = Option<<T as Typed>::ValueType>> {}
