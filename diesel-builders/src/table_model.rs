//! Submodule defining the `TableModel` trait.

use diesel::Table;
use tuplities::prelude::TuplePopFront;

use crate::{GetColumn, GetColumns, HasTableAddition, TableAddition};

/// Trait representing a Diesel table model.
pub trait TableModel:
    HasTableAddition<Table: TableAddition<Model = Self>>
    + GetColumns<<Self::Table as Table>::AllColumns>
    + GetColumns<<Self::Table as TableAddition>::PrimaryKeyColumns>
    + GetColumn<<<Self::Table as TableAddition>::PrimaryKeyColumns as TuplePopFront>::Front>
    + Sized
{
}

impl<T> TableModel for T where
    T: HasTableAddition<Table: TableAddition<Model = T>>
        + GetColumns<<T::Table as Table>::AllColumns>
        + GetColumns<<T::Table as TableAddition>::PrimaryKeyColumns>
        + GetColumn<<<T::Table as TableAddition>::PrimaryKeyColumns as TuplePopFront>::Front>
        + Sized
{
}
