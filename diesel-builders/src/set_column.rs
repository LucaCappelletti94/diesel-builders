//! Submodule providing the `SetColumn` trait.

use crate::{OptionalRef, Typed, TypedColumn};

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn> {
    /// Set the value of the specified column.
    fn set_column(&mut self, value: impl Into<Column::ColumnType>) -> &mut Self;
}

/// Trait providing a failable setter for a specific Diesel column.
pub trait MaySetColumn<Column: TypedColumn>: SetColumn<Column> {
    #[inline]
    /// Set the value of the specified column if the value is present.
    fn may_set_column(&mut self, value: Option<Column::ColumnType>) -> &mut Self {
        if let Some(v) = value {
            <Self as SetColumn<Column>>::set_column(self, v);
        }
        self
    }
}

impl<T, Column> MaySetColumn<Column> for T
where
    T: SetColumn<Column>,
    Column: TypedColumn,
{
}

/// Trait validating a specific Diesel column.
pub trait ValidateColumn<C: Typed> {
    /// The associated error type for the operation.
    type Error;

    #[inline]
    /// Validate the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value is invalid.
    fn validate_column(_value: &C::ValueType) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    /// Validate the value of the specified column, given the context of the entire
    /// new record being built.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value is invalid.
    fn validate_column_in_context(&self, value: &C::ValueType) -> Result<(), Self::Error> {
        Self::validate_column(value)
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetColumn<C: Typed>: ValidateColumn<C> {
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column(&mut self, value: impl Into<C::ColumnType>)
    -> Result<&mut Self, Self::Error>;
}

impl<T, C> TrySetColumn<C> for (T,)
where
    Self: SetColumn<C> + ValidateColumn<C>,
    C: TypedColumn,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let value = value.into();
        if let Some(value_ref) = value.as_optional_ref() {
            <Self as ValidateColumn<C>>::validate_column_in_context(self, value_ref)?;
        }
        <Self as SetColumn<C>>::set_column(self, value);
        Ok(self)
    }
}

impl<Head, Tail, C> TrySetColumn<C> for (Head, Tail)
where
    Self: SetColumn<C> + ValidateColumn<C>,
    C: TypedColumn,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let value = value.into();
        if let Some(value_ref) = value.as_optional_ref() {
            <Self as ValidateColumn<C>>::validate_column_in_context(self, value_ref)?;
        }
        <Self as SetColumn<C>>::set_column(self, value);
        Ok(self)
    }
}

/// Extension trait for `SetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetColumnExt: Sized {
    #[inline]
    /// Set the value of the specified column.
    fn set_column_ref<Column>(&mut self, value: impl Into<Column::ColumnType>) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(self, value)
    }

    #[inline]
    #[must_use]
    /// Set the value of the specified column.
    fn set_column<Column>(mut self, value: impl Into<Column::ColumnType>) -> Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(&mut self, value);
        self
    }
}

impl<T> SetColumnExt for T {}

/// Extension trait for `TrySetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetColumnExt: Sized {
    #[inline]
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column_ref<Column>(
        &mut self,
        value: impl Into<Column::ColumnType>,
    ) -> Result<&mut Self, <Self as ValidateColumn<Column>>::Error>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(self, value)
    }

    #[inline]
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column<Column>(
        mut self,
        value: impl Into<Column::ColumnType>,
    ) -> Result<Self, <Self as ValidateColumn<Column>>::Error>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(&mut self, value)?;
        Ok(self)
    }
}

impl<T> TrySetColumnExt for T {}
