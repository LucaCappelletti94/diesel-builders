//! Submodule defining the `TableModel` trait.

use diesel::{Table, associations::HasTable};
use tuplities::prelude::*;

use crate::{
    Columns, GetColumn, GetColumns, HasTableAddition, NonCompositePrimaryKeyTables, TableAddition,
    Tables, table_addition::HasPrimaryKey, tables::TableModels,
};

/// Trait representing a Diesel table model.
pub trait TableModel:
    HasTableAddition<Table: TableAddition<Model = Self>>
    + GetColumns<<Self::Table as Table>::AllColumns>
    + Sized
    + Clone
    + core::fmt::Debug
    + 'static
{
}

impl<T> TableModel for T where
    T: HasTableAddition<Table: TableAddition<Model = T>>
        + GetColumns<<T::Table as Table>::AllColumns>
        + Sized
        + Clone
        + core::fmt::Debug
        + 'static
{
}

/// Trait representing a Diesel table model associated to a table
/// which has non-composite primary keys.
pub trait NonCompositePrimaryKeyTableModel:
    TableModel<Table: HasPrimaryKey<Model = Self>> + GetColumn<<Self::Table as Table>::PrimaryKey>
{
}

impl<T> NonCompositePrimaryKeyTableModel for T where
    T: TableModel<Table: HasPrimaryKey<Model = T>> + GetColumn<<T::Table as Table>::PrimaryKey>
{
}

/// Trait for tuples of non-composite primary key table models, providing access
/// to primary keys.
pub trait NonCompositePrimaryKeyTableModels:
    TableModels<Tables: NonCompositePrimaryKeyTables> + IntoTupleOption
{
    /// Get references to the primary keys of all models in the tuple.
    fn get_primary_keys(
        &self,
    ) -> <<<Self::Tables as Tables>::PrimaryKeys as Columns>::Types as TupleRef>::Ref<'_>;

    /// Get references to the primary keys of all optional models in the tuple.
    fn may_get_primary_keys(
        optional_self: &<Self as IntoTupleOption>::IntoOptions,
    ) -> <<<<Self::Tables as Tables>::PrimaryKeys as Columns>::Types as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions;
}

// Generate implementations for all tuple sizes (1-32)
#[allow(clippy::unused_unit)]
#[diesel_builders_macros::impl_table_model]
mod impls {}
