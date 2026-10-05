//! Blanket implementations for `GetColumn` on smart pointers.

use crate::{TypedColumn, get_column::GetColumn};

/// Forwards `GetColumn` through each transparent smart-pointer wrapper.
///
/// Every wrapper delegates identically via `(**self)`, which reaches the inner
/// `T` for `&T`, `Box<T>`, `Rc<T>`, and `Arc<T>` alike.
macro_rules! impl_get_column_for_wrapper {
    ($($wrapper:ty),+ $(,)?) => {
        $(
            impl<C, T> GetColumn<C> for $wrapper
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
        )+
    };
}

impl_get_column_for_wrapper!(&T, Box<T>, std::rc::Rc<T>, std::sync::Arc<T>);
