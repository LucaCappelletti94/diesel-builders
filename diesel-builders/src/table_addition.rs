//! Extended `Table` trait with additional functionality.

use tuplities::prelude::TupleRefFront;

use crate::{
    Columns, InsertableTableModel, TableModel, Typed, TypedColumn, columns::NonEmptyProjection,
};

/// Extended trait for Diesel tables.
pub trait TableAddition: diesel::Table<AllColumns: Columns, PrimaryKey: Typed> + Default {
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The associated insertable model for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
    /// The primary key columns of this table.
    type PrimaryKeyColumns: NonEmptyProjection<Table = Self>
        + TupleRefFront<Front: TypedColumn<Table = Self>>;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableAddition: diesel::associations::HasTable<Table: TableAddition> {}

impl<T> HasTableAddition for T where T: diesel::associations::HasTable<Table: TableAddition> {}
