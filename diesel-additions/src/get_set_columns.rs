//! Submodule providing the `GetColumns` trait.

use crate::{Columns, GetColumn, HomogeneousColumns, MayGetColumn, OptionTuple, TypedColumn};

/// Marker trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {
    /// Get the values of the specified columns.
    fn get_columns(&self) -> <CS::Types as crate::RefTuple>::Output<'_>;
}

impl<T> GetColumns<()> for T {
    fn get_columns(&self) -> () {
        ()
    }
}

/// Marker trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {
    /// May get the values of the specified columns.
    fn may_get_columns(
        &self,
    ) -> <<CS::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output;
}

impl<T> MayGetColumns<()> for T {
    fn may_get_columns(&self) -> () {
        ()
    }
}

/// Marker trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {
    /// Set the values of the specified columns.
    fn set_columns(
        &mut self,
        values: <<CS as Columns>::Types as crate::RefTuple>::Output<'_>,
    ) -> &mut Self;
}

impl<T> SetColumns<()> for T {
    fn set_columns(&mut self, _values: ()) -> &mut Self {
        self
    }
}

/// Marker trait indicating a builder can set multiple homogeneous columns.
pub trait SetHomogeneousColumn<Type, CS: HomogeneousColumns<Type>>: SetColumns<CS> {
    /// Set the values of the specified columns.
    fn set_homogeneous_columns(&mut self, value: &Type) -> &mut Self;
}

impl<T, Type> SetHomogeneousColumn<Type, ()> for T {
    fn set_homogeneous_columns(&mut self, _value: &Type) -> &mut Self {
        self
    }
}

/// Marker trait indicating a builder can try to set multiple columns.
pub trait TrySetColumns<CS: Columns> {
    /// Attempt to set the values of the specified columns.
    fn try_set_columns(
        &mut self,
        values: <<CS as Columns>::Types as crate::RefTuple>::Output<'_>,
    ) -> anyhow::Result<&mut Self>;
}

impl<T> TrySetColumns<()> for T {
    fn try_set_columns(&mut self, _values: ()) -> anyhow::Result<&mut Self> {
        Ok(self)
    }
}

/// Marker trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetColumns<CS: Columns> {
    /// Attempt to set the values of the specified columns.
    fn try_may_set_columns(
        &mut self,
        values: <<<CS as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output,
    ) -> anyhow::Result<&mut Self>;
}

impl<T> TryMaySetColumns<()> for T {
    fn try_may_set_columns(&mut self, _values: ()) -> anyhow::Result<&mut Self> {
        Ok(self)
    }
}

/// Marker trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetHomogeneousColumn<Type, CS: HomogeneousColumns<Type>>: TrySetColumns<CS> {
    /// Attempt to set the values of the specified columns.
    fn try_set_homogeneous_columns(&mut self, value: &Type) -> anyhow::Result<&mut Self>;
}

impl<T, Type> TrySetHomogeneousColumn<Type, ()> for T {
    fn try_set_homogeneous_columns(&mut self, _value: &Type) -> anyhow::Result<&mut Self> {
        Ok(self)
    }
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_get_columns]
mod impls {}
