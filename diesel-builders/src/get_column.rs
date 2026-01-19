//! Submodule providing the `GetColumn` trait.

use diesel::associations::HasTable;
use tuplities::prelude::{NestedTupleIndex, NestedTuplePopBack};

use crate::{AncestorOfIndex, ColumnTyped, DescendantOf, HasTableExt, TypedColumn};

/// Trait providing a getter for a specific Diesel column.
pub trait GetColumn<Column: ColumnTyped> {
    /// Get the value of the specified column.
    fn get_column_ref(&self) -> &Column::ColumnType;
    /// Get the owned value of the specified column.
    fn get_column(&self) -> Column::ColumnType {
        self.get_column_ref().clone()
    }
}

impl<T, C> GetColumn<C> for (T,)
where
    C: ColumnTyped,
    T: GetColumn<C>,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        self.0.get_column_ref()
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        self.0.get_column()
    }
}

impl<Head, Tail, C> GetColumn<C> for (Head, Tail)
where
    C: TypedColumn,
    Tail: NestedTuplePopBack<Back: HasTableExt<Table: DescendantOf<C::Table>>>,
    C::Table: AncestorOfIndex<<Tail::Back as HasTable>::Table>,
    (Head, Tail): NestedTupleIndex<
            <C::Table as AncestorOfIndex<<Tail::Back as HasTable>::Table>>::Idx,
            Element: GetColumn<C>,
        >,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        GetColumn::get_column_ref(self.nested_index())
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        GetColumn::get_column(self.nested_index())
    }
}

/// Trait providing a failable getter for a specific Diesel column.
pub trait MayGetColumn<C: ColumnTyped> {
    /// Get the reference of the specified column, returning `None` if not
    /// present.
    fn may_get_column_ref(&self) -> Option<&C::ColumnType>;
    /// Get the value of the specified column, returning `None` if not present.
    fn may_get_column(&self) -> Option<C::ColumnType> {
        self.may_get_column_ref().cloned()
    }
}

impl<T, C> MayGetColumn<C> for Option<T>
where
    C: ColumnTyped,
    T: GetColumn<C>,
{
    #[inline]
    fn may_get_column_ref(&self) -> Option<&C::ColumnType> {
        Some(self.as_ref()?.get_column_ref())
    }
}

/// Extension trait for [`GetColumn`] that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait GetColumnExt {
    /// Get a reference to the specified column.
    fn get_column_ref<Column>(&self) -> &Column::ColumnType
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column_ref(self)
    }

    /// Get the owned value of the specified column.
    fn get_column<Column>(&self) -> Column::ColumnType
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column(self)
    }
}

impl<T> GetColumnExt for T {}

/// Extension trait for [`MayGetColumn`] that allows specifying the column at
/// the method level.
pub trait MayGetColumnExt {
    /// Get a reference to specified column, returning `None` if not present.
    fn may_get_column_ref<'a, Column>(&'a self) -> Option<&'a Column::ColumnType>
    where
        Column: TypedColumn,
        Column::Table: 'a,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column_ref(self)
    }

    /// Get the owned value of the specified column, returning `None` if not
    /// present.
    fn may_get_column<Column>(&self) -> Option<Column::ColumnType>
    where
        Column: TypedColumn,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column(self)
    }
}

impl<T> MayGetColumnExt for T {}

mod blanket_impls;
pub mod dynamic;
pub use dynamic::TryGetDynamicColumn;

pub mod dynamic_multi;
pub use dynamic_multi::TryGetDynamicColumns;
