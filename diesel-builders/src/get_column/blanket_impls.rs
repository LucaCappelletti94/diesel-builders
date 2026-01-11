//! Blanket implementations for `GetColumn` on smart pointers.

use crate::TypedColumn;
use crate::get_column::GetColumn;

impl<C, T> GetColumn<C> for &T
where
    C: TypedColumn,
    T: GetColumn<C>,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        (*self).get_column_ref()
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        (*self).get_column()
    }
}

impl<C, T> GetColumn<C> for Box<T>
where
    C: TypedColumn,
    T: GetColumn<C>,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        (**self).get_column_ref()
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        (**self).get_column()
    }
}

impl<C, T> GetColumn<C> for std::rc::Rc<T>
where
    C: TypedColumn,
    T: GetColumn<C>,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        (**self).get_column_ref()
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        (**self).get_column()
    }
}

impl<C, T> GetColumn<C> for std::sync::Arc<T>
where
    C: TypedColumn,
    T: GetColumn<C>,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        (**self).get_column_ref()
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        (**self).get_column()
    }
}
