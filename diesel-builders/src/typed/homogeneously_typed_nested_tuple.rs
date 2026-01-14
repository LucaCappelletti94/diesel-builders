//! Submodule defining and implementing the `HomogeneouslyTypedNestedTuple`
//! trait.

use crate::{ColumnTyped, TypedNestedTuple};

/// Trait for recursive definition of homogeneous typed tuples.
/// All entries must have the same associated Type.
pub trait HomogeneouslyTypedNestedTuple<Type>: TypedNestedTuple {}

impl<Type> HomogeneouslyTypedNestedTuple<Type> for () {}

impl<Type, T> HomogeneouslyTypedNestedTuple<Type> for (T,) where T: ColumnTyped {}

impl<Type, Head, Tail> HomogeneouslyTypedNestedTuple<Type> for (Head, Tail)
where
    Head: ColumnTyped,
    Tail: HomogeneouslyTypedNestedTuple<Type>,
    Self: TypedNestedTuple<NestedTupleValueType = (Head::ValueType, Tail::NestedTupleValueType)>,
{
}
