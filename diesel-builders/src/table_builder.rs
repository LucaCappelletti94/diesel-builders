//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use std::{fmt::Debug, marker::PhantomData};

use diesel::{Column, Table, associations::HasTable};
use tuple_set::TupleSet;
use typed_tuple::prelude::{TypedFirst, TypedIndex};

use crate::{
    AncestorOfIndex, AncestralBuildableTable, BuilderBundles, BuilderError, BuilderResult,
    BundlableTable, BundlableTables, ClonableTuple, CompletedTableBuilderBundle, DebuggableTuple,
    DefaultTuple, DescendantOf, GetColumn, HorizontalSameAsKey, IncompleteBuilderError, Insert,
    InsertableTableModel, MayGetColumn, MayGetColumns, MaySetColumns, RecursiveInsert,
    SingletonForeignKey, TableAddition, TableBuilderBundle, TryMaySetColumns, TrySetColumn,
    TrySetHomogeneousColumn, TrySetMandatoryBuilder, TypedColumn, buildable_table::BuildableTable,
    table_addition::HasPrimaryKey, vertical_same_as_group::VerticalSameAsGroup,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable<AncestorsWithSelf: BundlableTables>> {
    /// The insertable models for the table and its ancestors.
    bundles: <T::AncestorsWithSelf as BundlableTables>::BuilderBundles,
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Clone for TableBuilder<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            bundles: self.bundles.clone_tuple(),
        }
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Debug for TableBuilder<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilder")
            .field("bundles", &self.bundles.debug_tuple())
            .finish()
    }
}

impl<T: BuildableTable> Default for TableBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self {
            bundles: DefaultTuple::default_tuple(),
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
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn may_get_column(&self) -> Option<&<C as TypedColumn>::Type> {
        use typed_tuple::prelude::TypedTuple;
        self.bundles.get().may_get_column()
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    type Error = <TableBuilderBundle<C::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        use typed_tuple::prelude::TypedTuple;
        self.bundles
            .map_mut(|builder_bundle| builder_bundle.try_set_column(value).map(|_| ()))?;
        // TODO: set vertical same-as columns in associated builders here.
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
        TypedIndex<<Key::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<Key::Table>>,
    <<T as TableAddition>::InsertableModel as InsertableTableModel>::Error: From<<<<Key as Column>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<Key as SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error> {
        use typed_tuple::prelude::TypedTuple;
        let columns = builder.may_get_columns();
        self.try_may_set_columns(columns)?;
        self.bundles.map_mut(|builder_bundle: &mut TableBuilderBundle<<Key as Column>::Table>| {
            builder_bundle
                .try_set_mandatory_builder(builder)
                .map(|_| ())
        })?;
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
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        use typed_tuple::prelude::TypedTuple;
        let columns = builder.may_get_columns();
        self.may_set_columns(columns);
        self.bundles.apply(|builder_bundle| {
            builder_bundle.set_mandatory_builder(builder);
        });
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
        TypedIndex<<Key::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<Key::Table>>,
    <<T as TableAddition>::InsertableModel as InsertableTableModel>::Error: From<<<<Key as Column>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>{
        use typed_tuple::prelude::TypedTuple;
        let columns = builder.may_get_columns();
        self.try_may_set_columns(columns)?;
        self.bundles.map_mut(|builder_bundle| {
            builder_bundle
                .try_set_discretionary_builder(builder)
                .map(|_| ())
        })?;
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
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        use typed_tuple::prelude::TypedTuple;
        let columns = builder.may_get_columns();
        self.may_set_columns(columns);
        self.bundles.apply(|builder_bundle| {
            builder_bundle.set_discretionary_builder(builder);
        });
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
