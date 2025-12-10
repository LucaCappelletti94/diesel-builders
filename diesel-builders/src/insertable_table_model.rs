//! Submodule defining the `TableModel` trait.

use crate::{HasTableExt, TableExt};

/// Trait representing an Insertable Diesel table model.
pub trait InsertableTableModel:
    HasTableExt<Table: TableExt<InsertableModel = Self>> + Default + diesel::Insertable<Self::Table>
{
    /// The higher-level validation error type for this insertable table model.
    type Error;
}
