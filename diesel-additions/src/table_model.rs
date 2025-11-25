//! Submodule defining the `TableModel` trait.

use diesel::Table;

use crate::{GetColumns, HasTableAddition, TableAddition};

/// Trait representing a Diesel table model.
pub trait TableModel:
    HasTableAddition<Table: TableAddition<Model = Self>>
    + GetColumns<<Self::Table as Table>::AllColumns>
    + Sized
    + Clone
    + 'static
{
}

impl<T> TableModel for T where
    T: HasTableAddition<Table: TableAddition<Model = T>>
        + GetColumns<<T::Table as Table>::AllColumns>
        + Sized
        + Clone
        + 'static
{
}
