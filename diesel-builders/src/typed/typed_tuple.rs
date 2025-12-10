//! Submodule providing the `TypedTuple` trait.

use tuplities::prelude::{FlattenNestedTuple, IntoTupleOption, NestTuple, TupleRef};

use crate::TypedNestedTuple;

/// Trait representing a tuple of objects with associated types.
pub trait TypedTuple: NestTuple {
    /// The Rust type associated with this tuple of objects.
    type TupleType: IntoTupleOption + TupleRef + NestTuple;
}

impl<T> TypedTuple for T
where
    T: NestTuple<Nested: TypedNestedTuple>,
{
    type TupleType =
        <<T::Nested as TypedNestedTuple>::NestedTupleType as FlattenNestedTuple>::Flattened;
}
