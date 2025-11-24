//! Submodule defining the `TableModel` trait.

use diesel::Column;
use typed_tuple::{TupleKey, TypedTuple};

use crate::{
    HasTableAddition, MayGetColumn, MayGetColumns, SetColumn, TableAddition, TrySetColumn,
    TrySetColumns, tables::InsertableTableModels,
};

/// Trait representing an Insertable Diesel table model.
pub trait InsertableTableModel:
    'static
    + HasTableAddition<Table: TableAddition<InsertableModel = Self>>
    + Default
    + diesel::Insertable<Self::Table>
    + MayGetColumns<Self::InsertableColumns>
    + TrySetColumns<Self::InsertableColumns>
{
    /// The subset of columns this model can insert into.
    type InsertableColumns: crate::Columns;
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
            model.set(value);
        });
    }
}

/// Try to set the value of a column in a tuple of insertable table models.
pub trait TrySetInsertableTableModelColumn<C: crate::TypedColumn>: InsertableTableModels {
    /// Try to set the value of the specified column.
    fn try_set(&mut self, value: &<C as crate::TypedColumn>::Type) -> anyhow::Result<()>;
}

impl<C, T> TrySetInsertableTableModelColumn<C> for T
where
    C: crate::TypedColumn + TupleKey<T>,
    T: InsertableTableModels
        + TypedTuple<
            <C as TupleKey<T>>::Idx,
            <<C as Column>::Table as TableAddition>::InsertableModel,
        >,
    <<C as Column>::Table as TableAddition>::InsertableModel: TrySetColumn<C>,
{
    fn try_set(&mut self, value: &<C as crate::TypedColumn>::Type) -> anyhow::Result<()> {
        self.get_mut().try_set(value)
    }
}

/// Get the value of a column from a tuple of insertable table models.
pub trait MayGetInsertableTableModelColumn<C: crate::TypedColumn>: InsertableTableModels {
    /// Get the value of the specified column.
    fn maybe_get(&self) -> Option<&<C as crate::TypedColumn>::Type>;
}

impl<C, T> MayGetInsertableTableModelColumn<C> for T
where
    C: crate::TypedColumn + TupleKey<T>,
    T: InsertableTableModels
        + TypedTuple<
            <C as TupleKey<T>>::Idx,
            <<C as Column>::Table as TableAddition>::InsertableModel,
        >,
    <<C as Column>::Table as TableAddition>::InsertableModel: MayGetColumn<C>,
{
    fn maybe_get(&self) -> Option<&<C as crate::TypedColumn>::Type> {
        self.get().maybe_get()
    }
}
