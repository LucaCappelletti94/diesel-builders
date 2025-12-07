//! Submodule defining the `TableModel` trait.

use crate::{HasTableAddition, TableAddition};

/// Trait representing an Insertable Diesel table model.
pub trait InsertableTableModel:
    'static
    + HasTableAddition<Table: TableAddition<InsertableModel = Self>>
    + Default
    + Clone
    + core::fmt::Debug
    + diesel::Insertable<Self::Table>
{
    /// The higher-level validation error type for this insertable table model.
    type Error;
}
