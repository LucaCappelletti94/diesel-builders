//! Submodule providing the `SetColumn` trait.

use crate::TypedColumn;

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn> {
    /// Set the value of the specified column.
    fn set_column(&mut self, value: &<Column as TypedColumn>::Type) -> &mut Self;
}

impl<C, T> SetColumn<C> for Option<T>
where
    C: crate::TypedColumn,
    T: SetColumn<C>,
{
    #[inline]
    fn set_column(&mut self, value: &<C as crate::TypedColumn>::Type) -> &mut Self {
        if let Some(inner) = self {
            inner.set_column(value);
        }
        self
    }
}

/// Trait providing a failable setter for a specific Diesel column.
pub trait MaySetColumn<Column: TypedColumn> {
    /// Set the value of the specified column if the value is present.
    fn may_set_column(&mut self, value: Option<&<Column as TypedColumn>::Type>) -> &mut Self;
}

impl<C, T> MaySetColumn<C> for Option<T>
where
    C: crate::TypedColumn,
    T: MaySetColumn<C>,
{
    #[inline]
    fn may_set_column(&mut self, value: Option<&<C as crate::TypedColumn>::Type>) -> &mut Self {
        if let Some(inner) = self {
            inner.may_set_column(value);
        }
        self
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetColumn<Column: TypedColumn> {
    /// Attempt to set the value of the specified column.
    fn try_set_column(
        &mut self,
        value: &<Column as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self>;
}

impl<C, T> TrySetColumn<C> for Option<T>
where
    C: crate::TypedColumn,
    T: TrySetColumn<C>,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: &<C as crate::TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self> {
        match self {
            Some(inner) => {
                inner.try_set_column(value)?;
            }
            None => {}
        }
        Ok(self)
    }
}

/// Extension trait for `SetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetColumnExt {
    /// Set the value of the specified column.
    fn set_column<Column>(&mut self, value: &<Column as TypedColumn>::Type) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>;
}

impl<T> SetColumnExt for T {
    #[inline]
    fn set_column<Column>(&mut self, value: &<Column as TypedColumn>::Type) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(self, value)
    }
}

/// Extension trait for `MaySetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait MaySetColumnExt {
    /// Set the value of the specified column if the value is present.
    fn may_set_column<Column>(
        &mut self,
        value: Option<&<Column as TypedColumn>::Type>,
    ) -> &mut Self
    where
        Column: TypedColumn,
        Self: MaySetColumn<Column>;
}

impl<T> MaySetColumnExt for T {
    #[inline]
    fn may_set_column<Column>(&mut self, value: Option<&<Column as TypedColumn>::Type>) -> &mut Self
    where
        Column: TypedColumn,
        Self: MaySetColumn<Column>,
    {
        <Self as MaySetColumn<Column>>::may_set_column(self, value)
    }
}

/// Extension trait for `TrySetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetColumnExt {
    /// Attempt to set the value of the specified column.
    fn try_set_column<Column>(
        &mut self,
        value: &<Column as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>;
}

impl<T> TrySetColumnExt for T {
    #[inline]
    fn try_set_column<Column>(
        &mut self,
        value: &<Column as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(self, value)
    }
}
