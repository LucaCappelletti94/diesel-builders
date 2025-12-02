//! Submodule providing the `SetColumn` trait.

use crate::TypedColumn;

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn>:
    TrySetColumn<Column, Error = core::convert::Infallible>
{
    /// Set the value of the specified column.
    fn set_column(&mut self, value: impl Into<<Column as TypedColumn>::Type>) -> &mut Self;
}

/// Trait providing a failable setter for a specific Diesel column.
pub trait MaySetColumn<Column: TypedColumn> {
    /// Set the value of the specified column if the value is present.
    fn may_set_column(&mut self, value: Option<&<Column as TypedColumn>::Type>) -> &mut Self;
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetColumn<Column: TypedColumn> {
    /// The associated error type for the operation.
    type Error: core::error::Error + Send + Sync;

    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column(
        &mut self,
        value: <Column as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error>;
}

/// Extension trait for `SetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetColumnExt: Sized {
    #[inline]
    /// Set the value of the specified column.
    fn set_column_ref<Column>(
        &mut self,
        value: impl Into<<Column as TypedColumn>::Type>,
    ) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(self, value)
    }

    #[inline]
    #[must_use]
    /// Set the value of the specified column.
    fn set_column<Column>(mut self, value: impl Into<<Column as TypedColumn>::Type>) -> Self
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
        value: impl TryInto<
            <Column as TypedColumn>::Type,
            Error: Into<<Self as TrySetColumn<Column>>::Error>,
        >,
    ) -> Result<&mut Self, <Self as TrySetColumn<Column>>::Error>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(self, value.try_into().map_err(Into::into)?)
    }

    #[inline]
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column<Column>(
        mut self,
        value: impl TryInto<
            <Column as TypedColumn>::Type,
            Error: Into<<Self as TrySetColumn<Column>>::Error>,
        >,
    ) -> Result<Self, <Self as TrySetColumn<Column>>::Error>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(
            &mut self,
            value.try_into().map_err(Into::into)?,
        )?;
        Ok(self)
    }
}

impl<T> TrySetColumnExt for T {}
