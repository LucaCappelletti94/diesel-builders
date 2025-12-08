//! Submodule providing the `GetColumn` trait.

use crate::{Typed, TypedColumn};

/// Trait providing a getter for a specific Diesel column.
pub trait GetColumn<Column: TypedColumn> {
    /// Get the value of the specified column.
    fn get_column_ref(&self) -> &<Column as Typed>::Type;
    /// Get the owned value of the specified column.
    fn get_column(&self) -> <Column as Typed>::Type {
        self.get_column_ref().clone()
    }
}

/// Trait providing a failable getter for a specific Diesel column.
pub trait MayGetColumn<C: TypedColumn> {
    /// Get the reference of the specified column, returning `None` if not present.
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a <C as Typed>::Type>
    where
        C::Table: 'a;
    /// Get the value of the specified column, returning `None` if not present.
    fn may_get_column(&self) -> Option<<C as Typed>::Type> {
        self.may_get_column_ref().cloned()
    }
}

impl<T, C> MayGetColumn<C> for Option<T>
where
    C: TypedColumn,
    T: GetColumn<C>,
{
    #[inline]
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a <C as Typed>::Type>
    where
        C::Table: 'a,
    {
        Some(self.as_ref()?.get_column_ref())
    }
    #[inline]
    fn may_get_column(&self) -> Option<<C as Typed>::Type> {
        Some(self.as_ref()?.get_column())
    }
}

/// Extension trait for `GetColumn` that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait GetColumnExt {
    /// Get a reference to the specified column.
    fn get_column_ref<Column>(&self) -> &<Column as Typed>::Type
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column_ref(self)
    }

    /// Get the owned value of the specified column.
    fn get_column<Column>(&self) -> <Column as Typed>::Type
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column(self)
    }
}

impl<T> GetColumnExt for T {}

/// Extension trait for `MayGetColumn` that allows specifying the column at the
/// method level.
pub trait MayGetColumnExt {
    /// Get a reference to specified column, returning `None` if not present.
    fn may_get_column_ref<'a, Column>(&'a self) -> Option<&'a <Column as Typed>::Type>
    where
        Column: TypedColumn,
        Column::Table: 'a,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column_ref(self)
    }

    /// Get the owned value of the specified column, returning `None` if not present.
    fn may_get_column<Column>(&self) -> Option<<Column as Typed>::Type>
    where
        Column: TypedColumn,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column(self)
    }
}

impl<T> MayGetColumnExt for T {}
