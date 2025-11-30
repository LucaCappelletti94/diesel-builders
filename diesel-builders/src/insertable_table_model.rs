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
    + TrySetColumns<<Self::Table as TableAddition>::InsertableColumns>
{
}

impl<T> InsertableTableModel for T where
    T: 'static
        + HasTableAddition<Table: TableAddition<InsertableModel = T>>
        + Default
        + Clone
        + core::fmt::Debug
        + diesel::Insertable<T::Table>
        + MayGetColumns<<T::Table as TableAddition>::InsertableColumns>
        + TrySetColumns<<T::Table as TableAddition>::InsertableColumns>
{
}
