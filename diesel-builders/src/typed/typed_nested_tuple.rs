//! Submodule defining and implementing the `TypedNestedTuple` trait.

use tuplities::prelude::{FlattenNestedTuple, IntoNestedTupleOption, NestedTupleRef};

use crate::{Typed, TypedTuple};

/// Trait for recursive definition of the `Typed` trait.
pub trait TypedNestedTuple: FlattenNestedTuple<Flattened: TypedTuple> {
    /// The associated nested type.
    type NestedTupleType: FlattenNestedTuple + IntoNestedTupleOption + NestedTupleRef;
}

impl TypedNestedTuple for () {
    type NestedTupleType = ();
}

impl<T> TypedNestedTuple for (T,)
where
    T: Typed,
    (T,): FlattenNestedTuple,
    (T::ColumnType,): FlattenNestedTuple,
{
    type NestedTupleType = (T::ColumnType,);
}

impl<Head, Tail> TypedNestedTuple for (Head, Tail)
where
    Head: Typed,
    Tail: TypedNestedTuple,
    (Head, Tail): FlattenNestedTuple,
    (Head::ColumnType, Tail::NestedTupleType): FlattenNestedTuple,
{
    type NestedTupleType = (Head::ColumnType, Tail::NestedTupleType);
}
