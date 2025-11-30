//! Submodule providing the `GetColumn` trait.

use crate::TypedColumn;

/// Trait providing a getter for a specific Diesel column.
pub trait GetColumn<Column: TypedColumn> {
    /// Get the value of the specified column.
    fn get_column(&self) -> &<Column as TypedColumn>::Type;
}

/// Trait providing a failable getter for a specific Diesel column.
pub trait MayGetColumn<Column: TypedColumn> {
    /// Get the value of the specified column, returning `None` if not present.
    fn may_get_column(&self) -> Option<&<Column as TypedColumn>::Type>;
}

impl<C, T> MayGetColumn<C> for Option<T>
where
    C: crate::TypedColumn,
    T: MayGetColumn<C>,
{
    #[inline]
    fn may_get_column(&self) -> Option<&<C as crate::TypedColumn>::Type> {
        match self {
            Some(inner) => inner.may_get_column(),
            None => None,
        }
    }
}

/// Extension trait for `GetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait GetColumnExt {
    /// Get the value of the specified column.
    fn get_column<Column>(&self) -> &<Column as TypedColumn>::Type
    where
        Column: TypedColumn,
        Self: GetColumn<Column>;
}

impl<T> GetColumnExt for T {
    #[inline]
    fn get_column<Column>(&self) -> &<Column as TypedColumn>::Type
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column(self)
    }
}

/// Extension trait for `MayGetColumn` that allows specifying the column at the
/// method level.
pub trait MayGetColumnExt {
    /// Get the value of the specified column, returning `None` if not present.
    fn may_get_column<Column>(&self) -> Option<&<Column as TypedColumn>::Type>
    where
        Column: TypedColumn,
        Self: MayGetColumn<Column>;
}

impl<T> MayGetColumnExt for T {
    #[inline]
    fn may_get_column<Column>(&self) -> Option<&<Column as TypedColumn>::Type>
    where
        Column: TypedColumn,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column(self)
    }
}
