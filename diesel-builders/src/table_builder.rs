//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use diesel::{Column, associations::HasTable};
use tuplities::prelude::*;

mod completed_table_builder;
mod core_traits;
mod serde;
pub use completed_table_builder::RecursiveBuilderInsert;

use crate::{
    AncestorOfIndex, BundlableTable, BundlableTables, DescendantOf, HorizontalSameAsKey,
    InsertableTableModel, MayGetColumn, MayGetColumns, MaySetColumns, SingletonForeignKey,
    TableAddition, TableBuilderBundle, TryMaySetColumns, TrySetColumn, TrySetMandatoryBuilder,
    Typed, TypedColumn, buildable_table::BuildableTable,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    bundles: <T::AncestorsWithSelf as BundlableTables>::BuilderBundles,
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
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndex<<C::Table as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn may_get_column(&self) -> Option<<C as Typed>::Type> {
        self.bundles.tuple_index().may_get_column()
    }

    #[inline]
    fn may_get_column_ref(&self) -> Option<&<C as Typed>::Type> {
        self.bundles.tuple_index().may_get_column_ref()
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleIndexMut<<C::Table as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<C::Table>>,
{
    type Error = <TableBuilderBundle<C::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.bundles.tuple_index_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<Key, T> TrySetMandatoryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: crate::MandatorySameAsIndex,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetColumns<<<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error, <Key as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<Key as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetMandatoryBuilder<
            Key,
            Table = Key::Table,
        >,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<Key::Table as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<Key::Table>>,
    <<T as TableAddition>::InsertableModel as InsertableTableModel>::Error: From<<<<Key as Column>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error> {
        let columns = builder.may_get_columns();
        self.try_may_set_columns(columns)?;
        self.bundles.tuple_index_mut().try_set_mandatory_builder(builder)?;
        Ok(self)
    }
}

impl<C, T> crate::SetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: crate::MandatorySameAsIndex,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<<C as HorizontalSameAsKey>::HostColumns>,
    TableBuilderBundle<C::Table>: crate::SetMandatoryBuilder<C>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleIndexMut<<C::Table as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_columns();
        self.may_set_columns(columns);
        self.bundles
            .tuple_index_mut()
            .set_mandatory_builder(builder);
        self
    }
}

impl<Key, T> crate::TrySetDiscretionaryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: crate::DiscretionarySameAsIndex,
    Key::Table: AncestorOfIndex<T> + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetColumns<
        <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error,
    <Key as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<Key as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<Key::Table>: crate::TrySetDiscretionaryBuilder<Key, Table = Key::Table>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<Key::Table as AncestorOfIndex<T>>::Idx, Element=TableBuilderBundle<Key::Table>>,
    <<T as TableAddition>::InsertableModel as InsertableTableModel>::Error: From<<<<Key as Column>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>{
        let columns = builder.may_get_columns();
        self.try_may_set_columns(columns)?;
        self.bundles.tuple_index_mut().try_set_discretionary_builder(builder)?;
        Ok(self)
    }
}

impl<C, T> crate::SetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: crate::DiscretionarySameAsIndex,
    C::Table: AncestorOfIndex<T> + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<<C as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<C::Table>: crate::SetDiscretionaryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleIndexMut<<C::Table as AncestorOfIndex<T>>::Idx, Element = TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        let columns = builder.may_get_columns();
        self.may_set_columns(columns);
        self.bundles
            .tuple_index_mut()
            .set_discretionary_builder(builder);
        self
    }
}
