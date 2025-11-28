//! Submodule providing the `GetColumns` trait.

use crate::{
    Columns, GetColumn, HomogeneousColumns, MayGetColumn, OptionTuple, SetColumn, TypedColumn,
};

/// Marker trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {
    /// Get the values of the specified columns.
    fn get(&self) -> <CS::Types as crate::RefTuple>::Output<'_>;
}

/// Marker trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {
    /// May get the values of the specified columns.
    fn may_get(&self) -> <<CS::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output;
}

/// Marker trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {
    /// Set the values of the specified columns.
    fn set(&mut self, values: <<CS as Columns>::Types as crate::RefTuple>::Output<'_>);
}

/// Marker trait indicating a builder can set multiple homogeneous columns.
pub trait SetHomogeneousColumn<CS: HomogeneousColumns> {
    /// Set the values of the specified columns.
    fn set(&mut self, value: &<CS as HomogeneousColumns>::Type);
}

/// Marker trait indicating a builder can try to set multiple columns.
pub trait TrySetColumns<CS: Columns> {
    /// Attempt to set the values of the specified columns.
    fn try_set(
        &mut self,
        values: <<CS as Columns>::Types as crate::RefTuple>::Output<'_>,
    ) -> anyhow::Result<()>;
}

/// Marker trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetColumns<CS: Columns> {
    /// Attempt to set the values of the specified columns.
    fn try_may_set(
        &mut self,
        values: <<<CS as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output,
    ) -> anyhow::Result<()>;
}

/// Marker trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetHomogeneousColumn<CS: HomogeneousColumns> {
    /// Attempt to set the values of the specified columns.
    fn try_set(&mut self, value: &<CS as HomogeneousColumns>::Type) -> anyhow::Result<()>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_get_columns]
mod impls {}
