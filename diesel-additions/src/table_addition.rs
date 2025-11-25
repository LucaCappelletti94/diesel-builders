//! Extended `Table` trait with additional functionality.

use diesel::Column;

use crate::{Columns, InsertableTableModel, TableModel};

/// Extended trait for Diesel tables.
pub trait TableAddition: 'static + diesel::Table<AllColumns: Columns> + Default {
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The associated insertable model for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableAddition: diesel::associations::HasTable<Table: TableAddition> {}

impl<T> HasTableAddition for T where T: diesel::associations::HasTable<Table: TableAddition> {}

/// Defines a table which has a non-composite primary key.
pub trait HasPrimaryKey: TableAddition<PrimaryKey: Column<Table = Self>> {}

impl<T> HasPrimaryKey for T where T: TableAddition<PrimaryKey: Column<Table = T>> {}