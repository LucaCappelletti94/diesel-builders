//! Submodule defining and implementing traits for non-composite primary key tables.

use crate::{
    Columns, HasPrimaryKeyColumn, TupleGetNestedColumns, TupleMayGetNestedColumns,
    columns::NestedColumns,
};
use tuplities::prelude::*;

use super::{NestedTables, Tables};

/// Trait for tables that have non-composite primary keys.
pub trait NonCompositePrimaryKeyTables: Tables<Nested: NonCompositePrimaryKeyNestedTables> {
    /// Tuple with the primary key column of each table.
    type PrimaryKeyColumns: Columns;
}

impl<T> NonCompositePrimaryKeyTables for T
where
    T: Tables<Nested: NonCompositePrimaryKeyNestedTables>,
{
    type PrimaryKeyColumns =
        <<T::Nested as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns as FlattenNestedTuple>::Flattened;
}

/// Trait for nested tables that have non-composite primary keys.
pub trait NonCompositePrimaryKeyNestedTables:
    NestedTables<
        Flattened: Tables,
        NestedModels: TupleGetNestedColumns<Self::NestedPrimaryKeyColumns>,
        OptionalNestedModels: TupleMayGetNestedColumns<Self::NestedPrimaryKeyColumns>,
    >
{
    /// Tuple with the primary key column of each table.
    type NestedPrimaryKeyColumns: NestedColumns<Flattened: Columns>;
}

impl NonCompositePrimaryKeyNestedTables for () {
    type NestedPrimaryKeyColumns = ();
}

impl<T> NonCompositePrimaryKeyNestedTables for (T,)
where
    T: HasPrimaryKeyColumn,
{
    type NestedPrimaryKeyColumns = (T::PrimaryKey,);
}

impl<Head, Tail> NonCompositePrimaryKeyNestedTables for (Head, Tail)
where
    Head: HasPrimaryKeyColumn,
    Tail: NonCompositePrimaryKeyNestedTables,
    Self: NestedTables<
            Flattened: Tables,
            NestedModels: TupleGetNestedColumns<(Head::PrimaryKey, Tail::NestedPrimaryKeyColumns)>,
            OptionalNestedModels: TupleMayGetNestedColumns<(
                Head::PrimaryKey,
                Tail::NestedPrimaryKeyColumns,
            )>,
        >,
    (Head::PrimaryKey, Tail::NestedPrimaryKeyColumns): NestedColumns,
    <<(Head::PrimaryKey, Tail::NestedPrimaryKeyColumns) as NestedColumns>::NestedTables as FlattenNestedTuple>::Flattened: NestTuple,
{
    type NestedPrimaryKeyColumns = (Head::PrimaryKey, Tail::NestedPrimaryKeyColumns);
}
