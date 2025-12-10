//! Submodule defining and implementing the `NestedColumnsCollection` trait.

use crate::TypedNestedTupleCollection;

use super::{ColumnsCollection, NestedColumns};

/// A trait representing a nested collection of nested Diesel columns.
pub trait NestedColumnsCollection:
    TypedNestedTupleCollection<FlattenedMatrix: ColumnsCollection>
{
}

impl NestedColumnsCollection for () {}

impl<C1: NestedColumns> NestedColumnsCollection for (C1,) {}

impl<Head, Tail> NestedColumnsCollection for (Head, Tail)
where
    Head: NestedColumns,
    (Head, Tail): TypedNestedTupleCollection,
    Tail: NestedColumnsCollection,
{
}
