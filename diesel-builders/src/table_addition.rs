//! Extended `Table` trait with additional functionality.

use crate::{InsertableTableModel, TableModel, columns::NonEmptyNestedProjection};

/// Extended trait for Diesel tables.
pub trait TableExt: diesel::Table + Default {
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The associated insertable model for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
    /// The primary key columns of this table.
    type NestedPrimaryKeyColumns: NonEmptyNestedProjection<Table = Self>;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableExt: diesel::associations::HasTable<Table: TableExt> {}

impl<T> HasTableExt for T where T: diesel::associations::HasTable<Table: TableExt> {}
