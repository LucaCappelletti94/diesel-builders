//! Submodule defining and implementing the `HomogeneouslyTypedNestedTuple`
//! trait.

use crate::{Typed, TypedNestedTuple};

/// Trait for recursive definition of homogeneous typed tuples.
/// All entries must have the same associated Type.
pub trait HomogeneouslyTypedNestedTuple<Type>: TypedNestedTuple {}

impl<Type> HomogeneouslyTypedNestedTuple<Type> for () {}

impl<Type, T> HomogeneouslyTypedNestedTuple<Type> for (T,) where T: Typed {}

impl<Type, Head, Tail> HomogeneouslyTypedNestedTuple<Type> for (Head, Tail)
where
    Head: Typed,
    Tail: HomogeneouslyTypedNestedTuple<Type>,
    Self: TypedNestedTuple<NestedTupleColumnType = (Head::ColumnType, Tail::NestedTupleColumnType)>,
{
}
