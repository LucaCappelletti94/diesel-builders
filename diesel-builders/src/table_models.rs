//! Submodule defining and implementing traits for tuples of TableModel structs.

use tuplities::prelude::*;

use crate::TableModel;

/// Trait for recursive definition of the `TableModels` trait.
pub trait NestedTableModels: FlattenNestedTuple + IntoNestedTupleOption {
    /// The associated nested tables.
    type NestedTables: FlattenNestedTuple;
}

impl NestedTableModels for () {
    type NestedTables = ();
}

impl<M> NestedTableModels for (M,)
where
    M: TableModel,
{
    type NestedTables = (M::Table,);
}

impl<Head, Tail> NestedTableModels for (Head, Tail)
where
    Head: TableModel,
    Tail: NestedTableModels,
    (Head, Tail): FlattenNestedTuple<Flattened: NestTuple<Nested = (Head, Tail)>>,
    (Head::Table, Tail::NestedTables): FlattenNestedTuple<Flattened: NestTuple>,
{
    type NestedTables = (Head::Table, Tail::NestedTables);
}
