//! Submodule defining the `TableModel` trait.

use crate::{HasTableAddition, MayGetColumns, TableAddition, TrySetColumns};

/// Trait representing an Insertable Diesel table model.
pub trait InsertableTableModel:
    'static
    + HasTableAddition<Table: TableAddition<InsertableModel = Self>>
    + Default
    + Clone
    + core::fmt::Debug
    + diesel::Insertable<Self::Table>
    + MayGetColumns<<Self::Table as TableAddition>::InsertableColumns>
    + TrySetColumns<Self::Error, <Self::Table as TableAddition>::InsertableColumns>
{
    /// The higher-level validation error type for this insertable table model.
    type Error: std::error::Error + Send + Sync;
}
