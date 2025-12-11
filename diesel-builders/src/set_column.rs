//! Submodule providing the `SetColumn` trait.

use core::convert::Infallible;

use crate::{Typed, TypedColumn};

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn>:
    TrySetColumn<Column, Error = core::convert::Infallible>
{
    #[inline]
    /// Set the value of the specified column.
    fn set_column(&mut self, value: impl Into<<Column as Typed>::Type>) -> &mut Self {
        // Safe to unwrap because the Error type is Infallible
        <Self as TrySetColumn<Column>>::try_set_column(self, value.into()).unwrap()
    }
}

impl<T, Column> SetColumn<Column> for T
where
    T: TrySetColumn<Column, Error = core::convert::Infallible>,
    Column: TypedColumn,
{
}

/// Trait providing an unchecked setter for a specific Diesel column.
///
/// This method is solely meant to be used in [`TableExt::NewValues`] impls
/// when writing a `TrySetColumn` impl. It allows to avoid the overhead of
/// accessing the i-th element of a nested tuple via typenum at each call site.
pub trait SetColumnUnchecked<Column: TypedColumn> {
    /// Set the value of the specified column.
    fn set_column_unchecked(&mut self, value: impl Into<<Column as Typed>::Type>) -> &mut Self;
}

impl<T, Column> SetColumnUnchecked<Column> for T
where
    T: SetColumn<Column>,
    Column: TypedColumn,
{
    #[inline]
    fn set_column_unchecked(&mut self, value: impl Into<<Column as Typed>::Type>) -> &mut Self {
        <Self as SetColumn<Column>>::set_column(self, value)
    }
}

/// Extension trait for `SetColumnUnchecked` that allows specifying the column at the
/// method level.
pub trait SetColumnUncheckedExt: Sized {
    #[inline]
    /// Set the value of the specified column.
    fn set_column_unchecked<Column>(
        &mut self,
        value: impl Into<<Column as Typed>::Type>,
    ) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumnUnchecked<Column>,
    {
        <Self as SetColumnUnchecked<Column>>::set_column_unchecked(self, value)
    }
}

impl<T> SetColumnUncheckedExt for T {}

/// Trait providing a failable setter for a specific Diesel column.
pub trait MaySetColumn<Column: TypedColumn>: SetColumn<Column> {
    #[inline]
    /// Set the value of the specified column if the value is present.
    fn may_set_column(&mut self, value: Option<<Column as Typed>::Type>) -> &mut Self {
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

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetColumn<C: Typed> {
    /// The associated error type for the operation.
    type Error: From<Infallible>;

    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error>;
}

/// Extension trait for `SetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetColumnExt: Sized {
    #[inline]
    /// Set the value of the specified column.
    fn set_column_ref<Column>(&mut self, value: impl Into<<Column as Typed>::Type>) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(self, value)
    }

    #[inline]
    #[must_use]
    /// Set the value of the specified column.
    fn set_column<Column>(mut self, value: impl Into<<Column as Typed>::Type>) -> Self
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
        value: impl TryInto<<Column as Typed>::Type, Error: Into<<Self as TrySetColumn<Column>>::Error>>,
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
        value: impl TryInto<<Column as Typed>::Type, Error: Into<<Self as TrySetColumn<Column>>::Error>>,
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
