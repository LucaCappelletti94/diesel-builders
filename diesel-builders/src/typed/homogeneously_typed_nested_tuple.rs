//! Submodule defining and implementing the `HomogeneouslyTypedNestedTuple` trait.

use crate::{Typed, TypedNestedTuple};

/// Trait for recursive definition of homogeneous typed tuples.
/// All entries must have the same associated Type.
pub trait HomogeneouslyTypedNestedTuple<Type>: TypedNestedTuple {}

impl<Type> HomogeneouslyTypedNestedTuple<Type> for () {}

impl<Type, T> HomogeneouslyTypedNestedTuple<Type> for (T,)
where
    T: Typed,
    T::ColumnType: From<Type>,
{
}

impl<Type, Head, Tail> HomogeneouslyTypedNestedTuple<Type> for (Head, Tail)
where
    Head: Typed,
    Head::ColumnType: From<Type>,
    Tail: HomogeneouslyTypedNestedTuple<Type>,
    Self: TypedNestedTuple<NestedTupleType = (Head::ColumnType, Tail::NestedTupleType)>,
{
}
