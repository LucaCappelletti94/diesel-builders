//! Submodule defining and implementing the `TypedNestedTuple` trait.

use tuplities::prelude::{
    FlattenNestedTuple, IntoNestedTupleOption, NestedTupleFrom, NestedTupleInto, NestedTupleRef,
};

use crate::ColumnTyped;

/// Trait for recursive definition of the `Typed` trait.
pub trait TypedNestedTuple: FlattenNestedTuple {
    /// The associated nested column type.
    type NestedTupleColumnType: FlattenNestedTuple
        + IntoNestedTupleOption
        + NestedTupleRef
        + NestedTupleInto<<Self::NestedTupleValueType as IntoNestedTupleOption>::IntoOptions>
        + NestedTupleFrom<Self::NestedTupleValueType>;
    /// The associated nested value type.
    type NestedTupleValueType: FlattenNestedTuple + IntoNestedTupleOption + NestedTupleRef;
}

impl TypedNestedTuple for () {
    type NestedTupleColumnType = ();
    type NestedTupleValueType = ();
}

impl<T> TypedNestedTuple for (T,)
where
    T: ColumnTyped,
    (T,): FlattenNestedTuple,
    (T::ColumnType,): FlattenNestedTuple,
{
    type NestedTupleColumnType = (T::ColumnType,);
    type NestedTupleValueType = (T::ValueType,);
}

impl<Head, Tail> TypedNestedTuple for (Head, Tail)
where
    Head: ColumnTyped,
    Tail: TypedNestedTuple,
    (Head, Tail): FlattenNestedTuple,
    (Head::ColumnType, Tail::NestedTupleColumnType): FlattenNestedTuple,
    (Head::ValueType, Tail::NestedTupleValueType): FlattenNestedTuple,
{
    type NestedTupleColumnType = (Head::ColumnType, Tail::NestedTupleColumnType);
    type NestedTupleValueType = (Head::ValueType, Tail::NestedTupleValueType);
}

/// Trait defining a nested n-uple of non-optionals.
pub trait NonOptionalTypedNestedTuple:
    TypedNestedTuple<NestedTupleColumnType = <Self as TypedNestedTuple>::NestedTupleValueType>
{
}

impl<T> NonOptionalTypedNestedTuple for T where
    T: TypedNestedTuple<NestedTupleColumnType = <T as TypedNestedTuple>::NestedTupleValueType>
{
}
