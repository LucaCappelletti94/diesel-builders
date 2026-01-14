//! Submodule defining and implementing the `HomogeneouslyTypedNestedColumns`
//! trait.

use super::NestedColumns;
use crate::{HomogeneouslyTypedNestedTuple, TypedColumn};

/// Trait representing a nested tuple of columns where all columns have the same
/// associated Type.
pub trait HomogeneouslyTypedNestedColumns<VT>:
    NestedColumns + HomogeneouslyTypedNestedTuple<VT>
{
}

impl<VT> HomogeneouslyTypedNestedColumns<VT> for () {}

impl<Type, C1: TypedColumn> HomogeneouslyTypedNestedColumns<Type> for (C1,) {}

impl<Type, Head, Tail> HomogeneouslyTypedNestedColumns<Type> for (Head, Tail)
where
    Head: TypedColumn,
    Tail: HomogeneouslyTypedNestedColumns<Type>,
    (Head, Tail): NestedColumns + HomogeneouslyTypedNestedTuple<Type>,
{
}
