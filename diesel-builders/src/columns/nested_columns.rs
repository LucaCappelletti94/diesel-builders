//! Submodule defining and implementing the `NestedColumns` trait.

use crate::{TypedColumn, TypedNestedTuple};
use tuplities::prelude::FlattenNestedTuple;

/// Trait representing a nested tuple of columns.
///
/// Given a tuple of columns `(C1, C2, C3, C4)`, the associated
/// nested columns would be `(C1, (C2, (C3, (C4,))))`.
pub trait NestedColumns: TypedNestedTuple + Default {
    /// Associated type representing a set of nested tables.
    type NestedTables: FlattenNestedTuple;
}

impl NestedColumns for () {
    type NestedTables = ();
}

impl<C1: TypedColumn> NestedColumns for (C1,) {
    type NestedTables = (C1::Table,);
}

impl<Head, Tail> NestedColumns for (Head, Tail)
where
    Head: TypedColumn,
    Tail: NestedColumns,
    (Head, Tail): TypedNestedTuple,
    (Head::Table, Tail::NestedTables): FlattenNestedTuple,
{
    type NestedTables = (Head::Table, Tail::NestedTables);
}
