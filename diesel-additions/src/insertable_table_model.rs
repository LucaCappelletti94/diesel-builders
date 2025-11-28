//! Submodule defining the `TableModel` trait.

use diesel::Column;
use typed_tuple::prelude::{TupleKey, TypedTuple};

use crate::{
    HasTableAddition, MayGetColumns, SetColumn, TableAddition, TrySetColumns,
    tables::InsertableTableModels,
};

/// Trait representing an Insertable Diesel table model.
pub trait InsertableTableModel:
    'static
    + HasTableAddition<Table: TableAddition<InsertableModel = Self>>
    + Default
    + diesel::Insertable<Self::Table>
    + MayGetColumns<<Self::Table as TableAddition>::InsertableColumns>
    + TrySetColumns<<Self::Table as TableAddition>::InsertableColumns>
{
}

impl<T> InsertableTableModel for T where
    T: 'static
        + HasTableAddition<Table: TableAddition<InsertableModel = T>>
        + Default
        + diesel::Insertable<T::Table>
        + MayGetColumns<<T::Table as TableAddition>::InsertableColumns>
        + TrySetColumns<<T::Table as TableAddition>::InsertableColumns>
{
}

/// Set the value of a column in a tuple of insertable table models.
pub trait SetInsertableTableModelColumn<C: crate::TypedColumn>: InsertableTableModels {
    /// Set the value of the specified column.
    fn set(&mut self, value: &<C as crate::TypedColumn>::Type);
}

impl<C, T> SetInsertableTableModelColumn<C> for T
where
    C: crate::TypedColumn + TupleKey<T>,
    T: InsertableTableModels
        + TypedTuple<
            <C as TupleKey<T>>::Idx,
            <<C as Column>::Table as TableAddition>::InsertableModel,
        >,
    <<C as Column>::Table as TableAddition>::InsertableModel: SetColumn<C>,
{
    fn set(&mut self, value: &<C as crate::TypedColumn>::Type) {
        self.apply(|model| {
            model.set_column(value);
        });
    }
}
