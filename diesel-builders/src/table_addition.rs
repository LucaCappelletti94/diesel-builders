//! Extended `Table` trait with additional functionality.

use tuplities::prelude::NestTuple;

use crate::{
    Columns, InsertableTableModel, TableModel,
    columns::{NonEmptyNestedProjection, NonEmptyProjection},
};

/// Extended trait for Diesel tables.
pub trait TableExt: diesel::Table<AllColumns: Columns> + Default {
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The associated insertable model for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
    /// The primary key columns of this table.
    type PrimaryKeyColumns: NonEmptyProjection<Table = Self, Nested: NonEmptyNestedProjection<Table = Self>>;
}

/// Extended trait for Diesel tables with nested primary key columns.
pub trait TableExt2: TableExt {
    /// The nested primary key columns of this table.
    type NestedPrimaryKeyColumns: NonEmptyNestedProjection<Table = Self>;
}

impl<T> TableExt2 for T
where
    T: TableExt,
{
    type NestedPrimaryKeyColumns = <T::PrimaryKeyColumns as NestTuple>::Nested;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableExt: diesel::associations::HasTable<Table: TableExt> {}

impl<T> HasTableExt for T where T: diesel::associations::HasTable<Table: TableExt> {}
