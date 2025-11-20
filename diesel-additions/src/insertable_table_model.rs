//! Submodule defining the `TableModel` trait.

use diesel::Table;

use crate::{GetColumns, HasTableAddition, MayGetColumns, TrySetColumns};

/// Trait representing an Insertable Diesel table model.
pub trait InsertableTableModel:
    HasTableAddition
    + Default
    + diesel::Insertable<Self::Table>
    + MayGetColumns<Self::InsertableColumns>
    + TrySetColumns<Self::InsertableColumns>
{
    /// The subset of columns this model can insert into.
    type InsertableColumns: crate::Columns;
}

/// Trait representing a Diesel table that has an associated insertable model.
pub trait InsertableTable: Table {
    /// The associated insertable model type for this table.
    type InsertableModel: InsertableTableModel<Table = Self>;
}
