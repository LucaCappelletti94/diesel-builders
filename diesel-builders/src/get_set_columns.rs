//! Submodule providing the `GetColumns` trait.

use diesel::associations::HasTable;

use crate::{
    Columns, GetColumn, HasTableAddition, HomogeneousColumns, InsertableTableModel, MayGetColumn,
    OptionTuple, RefTuple, TableAddition, TrySetColumn, TypedColumn,
};

/// Marker trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {
    /// Get the values of the specified columns.
    fn get_columns(&self) -> <CS::Types as crate::RefTuple>::Output<'_>;
}

/// Doc test for empty tuple implementation.
///
/// # Examples
///
/// ```
/// use diesel_builders::GetColumns;
///
/// struct MyBuilder;
///
/// let builder = MyBuilder;
/// // Empty tuple columns can be retrieved
/// builder.get_columns();
/// ```
impl<T> GetColumns<()> for T {
    #[inline]
    fn get_columns(&self) {}
}

/// Marker trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {
    /// May get the values of the specified columns.
    fn may_get_columns(
        &self,
    ) -> <<CS::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output;
}

/// Doc test for empty tuple implementation.
///
/// # Examples
///
/// ```
/// use diesel_builders::MayGetColumns;
///
/// struct MyBuilder;
///
/// let builder = MyBuilder;
/// // Empty tuple columns can be optionally retrieved
/// builder.may_get_columns();
/// ```
impl<T> MayGetColumns<()> for T {
    #[inline]
    fn may_get_columns(&self) {}
}

/// Marker trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {
    /// Set the values of the specified columns.
    fn set_columns(
        &mut self,
        values: <<CS as Columns>::Types as crate::RefTuple>::Output<'_>,
    ) -> &mut Self;
}

/// Doc test for empty tuple implementation.
///
/// # Examples
///
/// ```
/// use diesel_builders::SetColumns;
///
/// struct MyBuilder;
///
/// let mut builder = MyBuilder;
/// // Empty tuple columns can be set
/// builder.set_columns(());
/// ```
impl<T> SetColumns<()> for T {
    #[inline]
    fn set_columns(&mut self, _values: ()) -> &mut Self {
        self
    }
}

/// Marker trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: Columns> {
    /// May set the values of the specified columns.
    fn may_set_columns(
        &mut self,
        values: <<<CS as Columns>::Types as crate::RefTuple>::Output<'_> as OptionTuple>::Output,
    ) -> &mut Self;
}

/// Doc test for empty tuple implementation.
///
/// # Examples
///
/// ```
/// use diesel_builders::MaySetColumns;
///
/// struct MyBuilder;
///
/// let mut builder = MyBuilder;
/// // Empty tuple columns can be optionally set
/// builder.may_set_columns(());
/// ```
impl<T> MaySetColumns<()> for T {
    #[inline]
    fn may_set_columns(&mut self, _values: ()) -> &mut Self {
        self
    }
}

/// Marker trait indicating a builder can set multiple homogeneous columns.
pub trait SetHomogeneousColumn<Type, CS: HomogeneousColumns<Type>>: SetColumns<CS> {
    /// Set the values of the specified columns.
    fn set_homogeneous_columns(&mut self, value: &Type) -> &mut Self;
}

/// Doc test for empty tuple implementation.
///
/// # Examples
///
/// ```
/// use diesel_builders::SetHomogeneousColumn;
///
/// struct MyBuilder;
///
/// let mut builder = MyBuilder;
/// let value = 42i32;
/// // Empty tuple homogeneous columns can be set
/// builder.set_homogeneous_columns(&value);
/// ```
impl<T, Type> SetHomogeneousColumn<Type, ()> for T {
    #[inline]
    fn set_homogeneous_columns(&mut self, _value: &Type) -> &mut Self {
        self
    }
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
        values: <<CS as Columns>::Types as RefTuple>::Output<'_>,
    ) -> Result<&mut Self, Error>;
}

impl<Error, T> TrySetColumns<Error, ()> for T {
    #[inline]
    fn try_set_columns(&mut self, _values: ()) -> Result<&mut Self, Error> {
        Ok(self)
    }
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
        values: <<<CS as Columns>::Types as RefTuple>::Output<'_> as OptionTuple>::Output,
    ) -> Result<&mut Self, Error>;
}

impl<Error, T> TryMaySetColumns<Error, ()> for T {
    #[inline]
    fn try_may_set_columns(&mut self, _values: ()) -> Result<&mut Self, Error> {
        Ok(self)
    }
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

impl<T: HasTableAddition, Error, Type> TrySetHomogeneousColumn<Error, Type, ()> for T {
    #[inline]
    fn try_set_homogeneous_columns(&mut self, _value: &Type) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_get_columns]
mod impls {}
