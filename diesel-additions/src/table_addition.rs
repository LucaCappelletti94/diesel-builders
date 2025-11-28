//! Extended `Table` trait with additional functionality.

use crate::{InsertableTableModel, Projection, TableModel, TypedColumn};

/// Extended trait for Diesel tables.
pub trait TableAddition: 'static + diesel::Table<AllColumns: Projection> + Default {
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The associated insertable model for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableAddition: diesel::associations::HasTable<Table: TableAddition> {}

impl<T> HasTableAddition for T where T: diesel::associations::HasTable<Table: TableAddition> {}

/// Defines a table which has a non-composite primary key.
pub trait HasPrimaryKey: TableAddition<PrimaryKey: TypedColumn<Table = Self>> {}

impl<T> HasPrimaryKey for T where T: TableAddition<PrimaryKey: TypedColumn<Table = T>> {}
