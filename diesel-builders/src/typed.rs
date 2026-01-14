//! Submodule providing the `Typed` trait.

mod typed_nested_tuple;
use std::fmt::Debug;

pub use typed_nested_tuple::*;
mod typed_nested_tuple_collection;
pub use typed_nested_tuple_collection::*;
mod homogeneously_typed_nested_tuple;
pub use homogeneously_typed_nested_tuple::*;

/// Trait representing an object with an associated value type.
pub trait ValueTyped {
    /// The value type associated with this object.
    type ValueType: Clone;
}

/// Trait representing an object with an associated type.
///
/// Extends [`ValueTyped`].
pub trait ColumnTyped: ValueTyped {
    /// The column type associated with this object, which may be an `Option` of
    /// the value type.
    type ColumnType: Clone
        + Debug
        + From<Self::ValueType>
        + Into<Option<Self::ValueType>>
        + OptionalRef<Self::ValueType>;
}

impl<C: ValueTyped + ?Sized> ValueTyped for Box<C> {
    type ValueType = C::ValueType;
}

impl<C: ColumnTyped + ?Sized> ColumnTyped for Box<C> {
    type ColumnType = C::ColumnType;
}

impl<C: ValueTyped + ?Sized> ValueTyped for &C {
    type ValueType = C::ValueType;
}

impl<C: ColumnTyped + ?Sized> ColumnTyped for &C {
    type ColumnType = C::ColumnType;
}

/// Trait providing a method to get an optional reference to another type.
pub trait OptionalRef<Other> {
    /// Get an optional reference to the other type.
    fn as_optional_ref(&self) -> Option<&Other>;
}

impl<T> OptionalRef<T> for T {
    fn as_optional_ref(&self) -> Option<&T> {
        Some(self)
    }
}

impl<T> OptionalRef<T> for Option<T> {
    fn as_optional_ref(&self) -> Option<&T> {
        self.as_ref()
    }
}

/// Trait representing an object whose `ValueType` and `ColumnType` are the
/// same, and therefore cannot be optional.
///
/// Extends [`ColumnTyped`].
pub trait NonOptionalTyped: ColumnTyped<ValueType = <Self as ColumnTyped>::ColumnType> {}

impl<T> NonOptionalTyped for T where T: ColumnTyped<ValueType = <T as ColumnTyped>::ColumnType> {}

/// Trait representing an object whose `ColumnType` is an `Option` of its
/// `ValueType`.
///
/// Extends [`ColumnTyped`].
pub trait OptionalTyped: ColumnTyped<ColumnType = Option<<Self as ValueTyped>::ValueType>> {}

impl<T> OptionalTyped for T where T: ColumnTyped<ColumnType = Option<<T as ValueTyped>::ValueType>> {}
