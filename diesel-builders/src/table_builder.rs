//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use std::{fmt::Debug, marker::PhantomData};

use diesel::{Table, associations::HasTable};
use diesel_additions::{
    ClonableTuple, DebuggableTuple, DefaultTuple, GetColumn, MayGetColumn, SetColumn,
    TableAddition, TrySetColumn, TrySetHomogeneousColumn, TypedColumn,
    table_addition::HasPrimaryKey,
};
use diesel_relations::{
    AncestorOfIndex, DescendantOf, vertical_same_as_group::VerticalSameAsGroup,
};
use tuple_set::TupleSet;
use typed_tuple::prelude::{TypedFirst, TypedTuple};

use crate::{
    AncestralBuildableTable, BuilderBundles, BundlableTable, BundlableTables,
    CompletedTableBuilderBundle, NestedInsert, TableBuilderBundle, TrySetMandatoryBuilder,
    buildable_table::BuildableTable,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable<AncestorsWithSelf: BundlableTables>> {
    /// The insertable models for the table and its ancestors.
    bundles: <T::AncestorsWithSelf as BundlableTables>::BuilderBundles,
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Clone for TableBuilder<T> {
    fn clone(&self) -> Self {
        Self { bundles: self.bundles.clone_tuple() }
    }
}

impl<T: BuildableTable<AncestorsWithSelf: BundlableTables>> Debug for TableBuilder<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilder").field("bundles", &self.bundles.debug_tuple()).finish()
    }
}

impl<T: BuildableTable> Default for TableBuilder<T> {
    fn default() -> Self {
        Self { bundles: DefaultTuple::default_tuple() }
    }
}

/// A completed builder for creating insertable models for a Diesel table and
/// its ancestors.
struct CompletedTableBuilder<T: Table, Bundles> {
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

    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, Bundles> HasTable for CompletedTableBuilder<T, Bundles>
where
    T: BuildableTable,
{
    type Table = T;

    fn table() -> Self::Table {
        T::default()
    }
}

impl<T> TryFrom<TableBuilder<T>>
    for CompletedTableBuilder<T, <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles>
where
    T: BuildableTable,
{
    type Error = anyhow::Error;

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
        Ok(CompletedTableBuilder { bundles, table: PhantomData })
    }
}

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: SetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn set_column(&mut self, value: &<C as TypedColumn>::Type) -> &mut Self {
        self.bundles.apply(|builder_bundle| {
            builder_bundle.set_column(value);
        });
        // TODO: set vertical same-as columns in associated builders here.
        self
    }
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: MayGetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn may_get_column(&self) -> Option<&<C as TypedColumn>::Type> {
        self.bundles.get().may_get_column()
    }
}

impl<C, T, Bundles> SetColumn<C> for CompletedTableBuilder<T, Bundles>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: SetColumn<C>,
    // We require for the non-completed variant of the builder
    // to implement SetColumn as well so to have a compile-time
    // verification of the availability of the column which
    // the `TupleSet` dynamic trait cannot guarantee.
    TableBuilder<T>: SetColumn<C>,
    Bundles: TupleSet,
{
    fn set_column(&mut self, value: &<C as TypedColumn>::Type) -> &mut Self {
        self.bundles.map(|builder_bundle: &mut CompletedTableBuilderBundle<C::Table>| {
            builder_bundle.set_column(value);
        });
        // TODO: set vertical same-as columns in associated builders here.
        self
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: TrySetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn try_set_column(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<&mut Self> {
        self.bundles.map_mut(|builder_bundle| builder_bundle.try_set_column(value).map(|_| ()))?;
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
    fn try_set_column(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<&mut Self> {
        self.bundles
            .map(|builder_bundle: &mut CompletedTableBuilderBundle<C::Table>| {
                builder_bundle.try_set_column(value).map(|_| ())
            })
            .transpose()?;
        // TODO: set vertical same-as columns in associated builders here.
        Ok(self)
    }
}

impl<C, T> TrySetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: diesel_relations::MandatorySameAsIndex,
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    TableBuilderBundle<C::Table>: TrySetMandatoryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self> {
        self.bundles.map_mut(|builder_bundle| {
            builder_bundle.try_set_mandatory_builder(builder.clone()).map(|_| ())
        })?;
        Ok(self)
    }
}

impl<C, T> crate::SetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: diesel_relations::MandatorySameAsIndex,
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    TableBuilderBundle<C::Table>: crate::SetMandatoryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        self.bundles.apply(|builder_bundle| {
            builder_bundle.set_mandatory_builder(builder.clone());
        });
        self
    }
}

impl<C, T> crate::TrySetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: diesel_relations::DiscretionarySameAsIndex,
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    TableBuilderBundle<C::Table>: crate::TrySetDiscretionaryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self> {
        self.bundles.map_mut(|builder_bundle| {
            builder_bundle.try_set_discretionary_builder(builder.clone()).map(|_| ())
        })?;
        Ok(self)
    }
}

impl<C, T> crate::SetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: diesel_relations::DiscretionarySameAsIndex,
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    TableBuilderBundle<C::Table>: crate::SetDiscretionaryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedTuple<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>,
    ) -> &mut Self {
        self.bundles.apply(|builder_bundle| {
            builder_bundle.set_discretionary_builder(builder.clone());
        });
        self
    }
}

impl<Conn, T> NestedInsert<Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilder<T, <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles>:
        NestedInsert<Conn, Table = T>,
{
    fn insert(&self, conn: &mut Conn) -> anyhow::Result<<Self::Table as TableAddition>::Model> {
        let completed_builder: CompletedTableBuilder<
            T,
            <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles,
        > = self.clone().try_into()?;
        completed_builder.insert(conn)
    }
}

// Base case: single bundle (leaf node)
impl<Conn, T> NestedInsert<Conn> for CompletedTableBuilder<T, (CompletedTableBuilderBundle<T>,)>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilderBundle<T>: NestedInsert<Conn, Table = T>,
{
    fn insert(&self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        Ok(self.bundles.0.insert(conn)?)
    }
}

// Recursive cases for tuples of size 2-32 are generated by the macro
#[diesel_builders_macros::impl_completed_table_builder_nested_insert]
mod completed_table_builder_impls {}
