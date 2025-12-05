//! Extended `Table` trait with additional functionality.

use diesel::query_dsl::methods::SelectDsl;
use typenum::U0;

use crate::{IndexedColumn, InsertableTableModel, Projection, TableModel};

/// Extended trait for Diesel tables.
pub trait TableAddition:
    'static + diesel::Table<AllColumns: Projection<Self>> + Default + SelectDsl<Self::AllColumns>
{
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The associated insertable model for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
    /// The set of insertable columns for this table.
    type InsertableColumns: Projection<Self>;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableAddition: diesel::associations::HasTable<Table: TableAddition> {}

impl<T> HasTableAddition for T where T: diesel::associations::HasTable<Table: TableAddition> {}

/// Defines a table which has a non-composite primary key.
pub trait HasPrimaryKey:
    TableAddition<PrimaryKey: IndexedColumn<U0, (Self::PrimaryKey,), Table = Self>>
{
}

impl<T> HasPrimaryKey for T where
    T: TableAddition<PrimaryKey: IndexedColumn<U0, (Self::PrimaryKey,), Table = Self>>
{
}
