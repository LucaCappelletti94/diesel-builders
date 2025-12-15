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
    (T::NestedTupleColumnType,): FlattenNestedTupleMatrix,
{
    type NestedCollectionType = (T::NestedTupleColumnType,);
}

impl<Head, Tail> TypedNestedTupleCollection for (Head, Tail)
where
    Head: TypedNestedTuple,
    Tail: TypedNestedTupleCollection,
    (Head, Tail): FlattenNestedTupleMatrix,
    (Head::NestedTupleColumnType, Tail::NestedCollectionType): FlattenNestedTupleMatrix,
{
    type NestedCollectionType = (Head::NestedTupleColumnType, Tail::NestedCollectionType);
}
