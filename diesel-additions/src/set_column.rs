//! Submodule providing the `SetColumn` trait.

use crate::TypedColumn;

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn> {
    /// Set the value of the specified column.
    fn set_column(&mut self, value: &<Column as TypedColumn>::Type);
}

impl<C, T> SetColumn<C> for Option<T>
where
    C: crate::TypedColumn,
    T: SetColumn<C>,
{
    fn set_column(&mut self, value: &<C as crate::TypedColumn>::Type) {
        if let Some(inner) = self {
            inner.set_column(value);
        }
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetColumn<Column: TypedColumn> {
    /// Attempt to set the value of the specified column.
    fn try_set_column(&mut self, value: &<Column as TypedColumn>::Type) -> anyhow::Result<()>;
}

impl<C, T> TrySetColumn<C> for Option<T>
where
    C: crate::TypedColumn,
    T: TrySetColumn<C>,
{
    fn try_set_column(&mut self, value: &<C as crate::TypedColumn>::Type) -> anyhow::Result<()> {
        match self {
            Some(inner) => inner.try_set_column(value),
            None => Ok(()),
        }
    }
}
