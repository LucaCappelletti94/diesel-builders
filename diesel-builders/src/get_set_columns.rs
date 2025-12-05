//! Submodule providing the `GetColumns` trait.

use crate::{
    Columns, GetColumn, HasTableAddition, HomogeneousColumns, InsertableTableModel, MayGetColumn,
    TableAddition, TrySetColumn, TypedColumn,
};
use diesel::associations::HasTable;
use tuplities::prelude::*;

/// Marker trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {
    /// Get the values of the specified columns.
    fn get_columns(&self) -> <CS::Types as TupleRef>::Ref<'_>;
}

/// Marker trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {
    /// May get the values of the specified columns.
    fn may_get_columns(&self)
    -> <<CS::Types as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions;
}

/// Marker trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {
    /// Set the values of the specified columns.
    fn set_columns(&mut self, values: <<CS as Columns>::Types as TupleRef>::Ref<'_>) -> &mut Self;
}

/// Marker trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: Columns> {
    /// May set the values of the specified columns.
    fn may_set_columns(
        &mut self,
        values: <<<CS as Columns>::Types as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions,
    ) -> &mut Self;
}

/// Marker trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumns<Error, CS: Columns> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_columns(
        &mut self,
        values: <<CS as Columns>::Types as TupleRef>::Ref<'_>,
    ) -> Result<&mut Self, Error>;
}

/// Marker trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetColumns<Error, CS: Columns> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_may_set_columns(
        &mut self,
        values: <<<CS as Columns>::Types as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error>;
}

/// Marker trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetHomogeneousColumn<Error, Type, CS: HomogeneousColumns<Type>>:
    TrySetColumns<Error, CS> + HasTableAddition
{
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_homogeneous_columns(&mut self, value: &Type) -> Result<&mut Self, Error>;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_get_columns]
#[diesel_builders_macros::impl_may_get_columns]
#[diesel_builders_macros::impl_set_columns]
#[diesel_builders_macros::impl_may_set_columns]
#[diesel_builders_macros::impl_try_set_columns]
#[diesel_builders_macros::impl_try_may_set_columns]
#[diesel_builders_macros::impl_try_set_homogeneous_column]
mod impls {}
