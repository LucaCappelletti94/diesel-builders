//! Submodule providing the `TypedNestedTupleCollection` trait.

use tuplities::prelude::*;

use crate::TypedNestedTuple;

/// Trait representing a nested collection of typed tuples.
pub trait TypedNestedTupleCollection: FlattenNestedTupleMatrix {
    /// The associated nested type.
    type NestedCollectionType: FlattenNestedTupleMatrix;
}

impl TypedNestedTupleCollection for () {
    type NestedCollectionType = ();
}

impl<T> TypedNestedTupleCollection for (T,)
where
    T: TypedNestedTuple,
    (T,): FlattenNestedTupleMatrix,
    (T::NestedTupleType,): FlattenNestedTupleMatrix,
{
    type NestedCollectionType = (T::NestedTupleType,);
}

impl<Head, Tail> TypedNestedTupleCollection for (Head, Tail)
where
    Head: TypedNestedTuple,
    Tail: TypedNestedTupleCollection,
    (Head, Tail): FlattenNestedTupleMatrix,
    (Head::NestedTupleType, Tail::NestedCollectionType): FlattenNestedTupleMatrix,
{
    type NestedCollectionType = (Head::NestedTupleType, Tail::NestedCollectionType);
}
