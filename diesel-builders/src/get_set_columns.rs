//! Submodule providing the `GetColumns` trait.

use crate::{
    Columns, GetColumn, HasTableAddition, InsertableTableModel, MayGetColumn, TableAddition,
    TrySetColumn, Typed, TypedColumn,
};
use diesel::associations::HasTable;
use tuplities::prelude::*;

#[diesel_builders_macros::impl_get_columns]
/// Trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {
    /// Get the values of the specified columns.
    fn get_columns(&self) -> <CS::Type as TupleRef>::Ref<'_>;
}

#[diesel_builders_macros::impl_tuple_get_columns]
/// Variant of `GetColumns` for n-uples.
pub trait TupleGetColumns<CS: Columns> {
    /// Get the values of the specified columns as an n-uple.
    fn tuple_get_columns(&self) -> <CS::Type as TupleRef>::Ref<'_>;
}

#[diesel_builders_macros::impl_may_get_columns]
/// Trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {
    /// May get the values of the specified columns.
    fn may_get_columns(&self) -> <<CS::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions;
}

#[diesel_builders_macros::impl_tuple_may_get_columns]
/// Variant of `MayGetColumns` for n-uples.
pub trait TupleMayGetColumns<CS: Columns> {
    /// May get the values of the specified columns as an n-uple.
    fn tuple_may_get_columns(
        &self,
    ) -> <<CS::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions;
}

/// Trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {
    /// Set the values of the specified columns.
    fn set_columns(&mut self, values: <<CS as Typed>::Type as TupleRef>::Ref<'_>) -> &mut Self;
}

/// Trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: Columns> {
    /// May set the values of the specified columns.
    fn may_set_columns(
        &mut self,
        values: <<<CS as Typed>::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions,
    ) -> &mut Self;
}

#[diesel_builders_macros::impl_try_set_columns]
/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumns<Error, CS: Typed<Type: TupleRef>> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_columns(
        &mut self,
        values: <<CS as Typed>::Type as TupleRef>::Ref<'_>,
    ) -> Result<&mut Self, Error>;
}

#[diesel_builders_macros::impl_try_set_columns_collection]
/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetColumnsCollection<Error, ColumnsCollection: Typed<Type: TupleRefMap>> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_columns_collection(
        &mut self,
        values: <<ColumnsCollection as Typed>::Type as TupleRefMap>::RefMap<'_>,
    ) -> Result<&mut Self, Error>;
}

/// Trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetColumns<Error, CS: Columns> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_may_set_columns(
        &mut self,
        values: <<<CS as Typed>::Type as TupleRef>::Ref<'_> as IntoTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error>;
}

/// Trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetHomogeneous<Error, Type, CS: Typed<Type: TupleRefMap>>:
    TrySetColumnsCollection<Error, CS>
{
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    fn try_set_homogeneous(&mut self, value: Type) -> Result<&mut Self, Error>;
}

impl<
    'a,
    Error,
    T,
    Type: core::fmt::Debug + Clone + 'a,
    ColumnsCollection: Typed<Type: TupleRefMap<RefMap<'a>: TupleReplicate<Type>>>,
> TrySetHomogeneous<Error, Type, ColumnsCollection> for T
where
    T: TrySetColumnsCollection<Error, ColumnsCollection>,
{
    #[inline]
    fn try_set_homogeneous(&mut self, value: Type) -> Result<&mut Self, Error> {
        let replicates =
            <<<ColumnsCollection as Typed>::Type as TupleRefMap>::RefMap<'_> as TupleReplicate<
                Type,
            >>::tuple_replicate(value);
        <T as TrySetColumnsCollection<Error, ColumnsCollection>>::try_set_columns_collection(
            self, replicates,
        )
    }
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_set_columns]
#[diesel_builders_macros::impl_may_set_columns]
#[diesel_builders_macros::impl_try_may_set_columns]
mod impls {}
