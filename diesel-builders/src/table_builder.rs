//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use std::{fmt::Debug, marker::PhantomData};

use diesel::{Column, Table, associations::HasTable};
use tuple_set::TupleSet;
use tuplities::prelude::*;

use crate::{
    AncestorOfIndex, AncestralBuildableTable, BuilderBundles, BuilderError, BuilderResult,
    BundlableTable, BundlableTables, CompletedTableBuilderBundle, DescendantOf, GetColumn,
    HorizontalSameAsKey, IncompleteBuilderError, Insert, InsertableTableModel, MayGetColumn,
    MayGetColumns, MaySetColumns, RecursiveInsert, SingletonForeignKey, TableAddition,
    TableBuilderBundle, TryMaySetColumns, TrySetColumn, TrySetHomogeneousColumn,
    TrySetMandatoryBuilder, TypedColumn, buildable_table::BuildableTable,
    table_addition::HasPrimaryKey, vertical_same_as_group::VerticalSameAsGroup,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable<AncestorsWithSelf: BundlableTables>> {
    /// The insertable models for the table and its ancestors.
    bundles: <T::AncestorsWithSelf as BundlableTables>::BuilderBundles,
}

#[cfg(feature = "serde")]
impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> serde::Serialize for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: serde::Serialize,
{
    #[inline]
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::ser::Serializer>::Ok, <S as serde::ser::Serializer>::Error>
    where
        S: serde::ser::Serializer,
    {
        self.bundles.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: BuildableTable<AncestorsWithSelf: BundlableTables>> serde::Deserialize<'de>
    for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let bundles =
            <T::AncestorsWithSelf as BundlableTables>::BuilderBundles::deserialize(deserializer)?;
        Ok(Self { bundles })
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Clone for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleClone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            bundles: self.bundles.tuple_clone(),
        }
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Copy for TableBuilder<T> where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: Copy + TupleCopy
{
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> PartialEq for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TuplePartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.bundles.tuple_eq(&other.bundles)
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Eq for TableBuilder<T> where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TuplePartialEq + TupleEq
{
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> std::hash::Hash for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleHash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bundles.tuple_hash(state);
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> PartialOrd for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TuplePartialOrd + TuplePartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bundles.tuple_partial_cmp(&other.bundles)
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Ord for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleOrd + TuplePartialOrd + TupleEq + TuplePartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bundles.tuple_cmp(&other.bundles)
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Debug for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleDebug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilder")
            .field("bundles", &self.bundles.tuple_debug())
            .finish()
    }
}

impl<T: BuildableTable> Default for TableBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self {
            bundles: TupleDefault::tuple_default(),
        }
    }
}

/// A completed builder for creating insertable models for a Diesel table and
/// its ancestors.
pub struct CompletedTableBuilder<T: Table, Bundles> {
    /// The insertable models for the table and its ancestors.
    bundles: Bundles,
    /// The table type.
    table: PhantomData<T>,
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

impl<T, Bundles> HasTable for CompletedTableBuilder<T, Bundles>
where
    T: BuildableTable,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T> TryFrom<TableBuilder<T>>
    for CompletedTableBuilder<T, <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles>
where
    T: BuildableTable,
{
    type Error = IncompleteBuilderError;

    #[inline]
    fn try_from(
        value: TableBuilder<T>,
    ) -> Result<
        CompletedTableBuilder<
            T,
            <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles,
        >,
        Self::Error,
    > {
        let bundles = value.bundles.try_complete()?;
        Ok(CompletedTableBuilder {
            bundles,
            table: PhantomData,
        })
    }
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: MayGetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndex<<C::Table as AncestorOfIndex<T>>::Idx, Type = TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn may_get_column(&self) -> Option<&<C as TypedColumn>::Type> {
        self.bundles.tuple_index().may_get_column()
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<C::Table as AncestorOfIndex<T>>::Idx, Type = TableBuilderBundle<C::Table>>,
{
    type Error = <TableBuilderBundle<C::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        self.bundles.tuple_index_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<C, T, Bundles> TrySetColumn<C> for CompletedTableBuilder<T, Bundles>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: TrySetColumn<C>,
    // We require for the non-completed variant of the builder
    // to implement TrySetColumn as well so to have a compile-time
    // verification of the availability of the column which
    // the `TupleSet` dynamic trait cannot guarantee.
    TableBuilder<T>: TrySetColumn<C>,
    Bundles: TupleSet,
{
    type Error = <CompletedTableBuilderBundle<C::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        <Bundles as TupleSet>::map(
            &mut self.bundles,
            |builder_bundle: &mut CompletedTableBuilderBundle<C::Table>| {
                builder_bundle.try_set_column(value).map(|_| ())
            },
        )
        .transpose()?;
        // TODO: set vertical same-as columns in associated builders here.
        Ok(self)
    }
}

impl<Key, T> TrySetMandatoryBuilder<Key> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<Key::Table>,
    Key: crate::MandatorySameAsIndex,
    Key::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetColumns<<<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error, <Key as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<Key as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<Key::Table>: TrySetMandatoryBuilder<
            Key,
            Table = Key::Table,
        >,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<Key::Table as AncestorOfIndex<T>>::Idx, Type = TableBuilderBundle<Key::Table>>,
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
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<<C as HorizontalSameAsKey>::HostColumns>,
    TableBuilderBundle<C::Table>: crate::SetMandatoryBuilder<C>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<C::Table as AncestorOfIndex<T>>::Idx, Type = TableBuilderBundle<C::Table>>,
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
    Key::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    Key::ReferencedTable: BuildableTable,
    Self: TryMaySetColumns<
        <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error,
    <Key as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<Key as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<Key::Table>: crate::TrySetDiscretionaryBuilder<Key, Table = Key::Table>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<Key::Table as AncestorOfIndex<T>>::Idx, Type=TableBuilderBundle<Key::Table>>,
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
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: MaySetColumns<<C as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<C::Table>: crate::SetDiscretionaryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TupleIndexMut<<C::Table as AncestorOfIndex<T>>::Idx, Type = TableBuilderBundle<C::Table>>,
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

impl<T, Conn> Insert<Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilder<T, <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles>:
        RecursiveInsert<
                <<T as TableAddition>::InsertableModel as InsertableTableModel>::Error,
                Conn,
                Table = T,
            >,
{
    #[inline]
    fn insert(self, conn: &mut Conn) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>{
        self.recursive_insert(conn)
    }
}

impl<T, Error, Conn> RecursiveInsert<Error, Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilder<T, <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles>:
        RecursiveInsert<Error, Conn, Table = T>,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error> {
        let completed_builder: CompletedTableBuilder<
            T,
            <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles,
        > = self.try_into()?;
        completed_builder.recursive_insert(conn)
    }
}

// Base case: single bundle (leaf node)
impl<Error, Conn, T> RecursiveInsert<Error, Conn>
    for CompletedTableBuilder<T, (CompletedTableBuilderBundle<T>,)>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilderBundle<T>: RecursiveInsert<Error, Conn, Table = T>,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error> {
        self.bundles.0.recursive_insert(conn)
    }
}

// Recursive cases for tuples of size 2-32 are generated by the macro
#[diesel_builders_macros::impl_completed_table_builder_nested_insert]
mod completed_table_builder_impls {}
