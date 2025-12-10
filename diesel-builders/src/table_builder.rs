//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use diesel::associations::HasTable;
use tuplities::prelude::*;

mod completed_table_builder;
// mod core_traits;
mod serde;
pub use completed_table_builder::{RecursiveBuilderInsert, RecursiveTableBuilder};

use crate::{
    AncestorOfIndex, BundlableTable, DescendantOf, DiscretionarySameAsIndex, InsertableTableModel,
    MandatorySameAsIndex, MayGetColumn, MayGetNestedColumns, MaySetColumns,
    SetDiscretionaryBuilder, SetMandatoryBuilder, SingletonForeignKey, TableBuilderBundle,
    TableExt, TryMaySetNestedColumns, TrySetColumn, TrySetDiscretionaryBuilder,
    TrySetMandatoryBuilder, Typed, TypedColumn, TypedNestedTuple, buildable_table::BuildableTable,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    bundles: T::NestedAncestorBuilders,
}

impl<T> HasTable for TableBuilder<T>
where
    T: BuildableTable,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: MayGetColumn<C>,
    T::NestedAncestorBuilders: NestedTupleIndex<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn may_get_column(&self) -> Option<<C as Typed>::Type> {
        self.bundles.nested_index().may_get_column()
    }

    #[inline]
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a <C as Typed>::Type>
    where
        C::Table: 'a,
    {
        self.bundles.nested_index().may_get_column_ref()
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    type Error = <TableBuilderBundle<C::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.bundles.nested_index_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<Key, T> TrySetMandatoryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: MandatorySameAsIndex,
    Key::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <Key::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetNestedColumns<
            <T::InsertableModel as InsertableTableModel>::Error,
            Key::NestedHostColumns,
        >,
    TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetMandatoryBuilder<Key, Table = Key::Table>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <Key::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<Key::Table>,
        >,
    <T::InsertableModel as InsertableTableModel>::Error:
        From<<<Key::Table as TableExt>::InsertableModel as InsertableTableModel>::Error>,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <T::InsertableModel as InsertableTableModel>::Error> {
        let columns = builder.may_get_nested_columns();
        self.try_may_set_nested_columns(columns)?;
        self.bundles
            .nested_index_mut()
            .try_set_mandatory_builder(builder)?;
        Ok(self)
    }
}

impl<C, T> SetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: MandatorySameAsIndex,
    C::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <C::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<C::NestedHostColumns>,
    TableBuilderBundle<C::Table>: SetMandatoryBuilder<C>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetNestedColumns<C::NestedForeignColumns>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_nested_columns();
        self.may_set_nested_columns(columns);
        self.bundles
            .nested_index_mut()
            .set_mandatory_builder(builder);
        self
    }
}

impl<Key, T> TrySetDiscretionaryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: DiscretionarySameAsIndex,
    Key::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <Key::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetNestedColumns<
            <T::InsertableModel as InsertableTableModel>::Error,
            Key::NestedHostColumns,
        >,
    TableBuilder<Key::ReferencedTable>: MayGetNestedColumns<Key::NestedForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetDiscretionaryBuilder<Key, Table = Key::Table>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <Key::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<Key::Table>,
        >,
    <T::InsertableModel as InsertableTableModel>::Error:
        From<<<Key::Table as TableExt>::InsertableModel as InsertableTableModel>::Error>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <T::InsertableModel as InsertableTableModel>::Error> {
        let columns = builder.may_get_nested_columns();
        self.try_may_set_nested_columns(columns)?;
        self.bundles
            .nested_index_mut()
            .try_set_discretionary_builder(builder)?;
        Ok(self)
    }
}

impl<C, T> SetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: DiscretionarySameAsIndex,
    C::NestedForeignColumns: TypedNestedTuple<
        NestedTupleType = <C::NestedHostColumns as TypedNestedTuple>::NestedTupleType,
    >,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<C::NestedHostColumns>,
    TableBuilder<C::ReferencedTable>: MayGetNestedColumns<C::NestedForeignColumns>,
    TableBuilderBundle<C::Table>: SetDiscretionaryBuilder<C>,
    T::NestedAncestorBuilders: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = TableBuilderBundle<C::Table>,
        >,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_nested_columns();
        self.may_set_nested_columns(columns);
        self.bundles
            .nested_index_mut()
            .set_discretionary_builder(builder);
        self
    }
}
