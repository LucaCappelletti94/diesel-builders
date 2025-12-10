//! Submodule defining and implementing the `HomogeneouslyTypedNestedColumns` trait.

use crate::{HomogeneouslyTypedNestedTuple, TypedColumn};

use super::NestedColumns;

/// Trait representing a nested tuple of columns where all columns have the same associated Type.
pub trait HomogeneouslyTypedNestedColumns<CT>:
    NestedColumns + HomogeneouslyTypedNestedTuple<CT>
{
}

impl<CT> HomogeneouslyTypedNestedColumns<CT> for () {}

impl<C1: TypedColumn> HomogeneouslyTypedNestedColumns<C1::Type> for (C1,) {}

impl<Head, Tail> HomogeneouslyTypedNestedColumns<Head::Type> for (Head, Tail)
where
    Head: TypedColumn,
    Tail: HomogeneouslyTypedNestedColumns<Head::Type>,
    (Head, Tail): NestedColumns + HomogeneouslyTypedNestedTuple<Head::Type>,
{
}
