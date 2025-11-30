//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use std::{fmt::Debug, marker::PhantomData};

use diesel::{Table, associations::HasTable};
use tuple_set::TupleSet;
use typed_tuple::prelude::{TypedFirst, TypedIndex};

use crate::{
    AncestorOfIndex, AncestralBuildableTable, BuilderBundles, BundlableTable, BundlableTables,
    ClonableTuple, CompletedTableBuilderBundle, DebuggableTuple, DefaultTuple, DescendantOf,
    GetColumn, HorizontalSameAsKey, MayGetColumn, MayGetColumns, MaySetColumn, MaySetColumns,
    NestedInsert, SetColumn, SingletonForeignKey, TableAddition, TableBuilderBundle,
    TryMaySetColumns, TrySetColumn, TrySetHomogeneousColumn, TrySetMandatoryBuilder, TypedColumn,
    buildable_table::BuildableTable, table_addition::HasPrimaryKey,
    vertical_same_as_group::VerticalSameAsGroup,
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
    type Error = anyhow::Error;

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

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: SetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn set_column(&mut self, value: &<C as TypedColumn>::Type) -> &mut Self {
        use typed_tuple::prelude::TypedTuple;
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
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn may_get_column(&self) -> Option<&<C as TypedColumn>::Type> {
        use typed_tuple::prelude::TypedTuple;
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
    #[inline]
    fn set_column(&mut self, value: &<C as TypedColumn>::Type) -> &mut Self {
        self.bundles.map(
            |builder_bundle: &mut CompletedTableBuilderBundle<C::Table>| {
                builder_bundle.set_column(value);
            },
        );
        // TODO: set vertical same-as columns in associated builders here.
        self
    }
}

impl<C, T> MaySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    TableBuilderBundle<C::Table>: MaySetColumn<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn may_set_column(&mut self, value: Option<&<C as TypedColumn>::Type>) -> &mut Self {
        use typed_tuple::prelude::TypedTuple;
        self.bundles.map_mut(|builder_bundle| {
            builder_bundle.may_set_column(value);
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
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn try_set_column(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        self.bundles
            .map_mut(|builder_bundle| builder_bundle.try_set_column(value).map(|_| ()))?;
        // TODO: set vertical same-as columns in associated builders here.
        Ok(self)
    }
}

impl<C, T, Bundles> MaySetColumn<C> for CompletedTableBuilder<T, Bundles>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: MaySetColumn<C>,
    // We require for the non-completed variant of the builder
    // to implement MaySetColumn as well so to have a compile-time
    // verification of the availability of the column which
    // the `TupleSet` dynamic trait cannot guarantee.
    TableBuilder<T>: MaySetColumn<C>,
    Bundles: TupleSet,
{
    #[inline]
    fn may_set_column(&mut self, value: Option<&<C as TypedColumn>::Type>) -> &mut Self {
        <Bundles as TupleSet>::map(
            &mut self.bundles,
            |builder_bundle: &mut CompletedTableBuilderBundle<C::Table>| {
                builder_bundle.may_set_column(value);
            },
        );
        // TODO: set vertical same-as columns in associated builders here.
        self
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
    #[inline]
    fn try_set_column(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<&mut Self> {
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

impl<C, T> TrySetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: crate::MandatorySameAsIndex,
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: TryMaySetColumns<<C as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<C::Table>: TrySetMandatoryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<<C as SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        let columns = builder.may_get_columns();
        self.try_may_set_columns(columns)?;
        self.bundles.map_mut(|builder_bundle| {
            builder_bundle
                .try_set_mandatory_builder(builder.clone())
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
            builder_bundle.set_mandatory_builder(builder.clone());
        });
        self
    }
}

impl<C, T> crate::TrySetDiscretionaryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: crate::DiscretionarySameAsIndex,
    C::Table: AncestorOfIndex<T> + BundlableTable + BuildableTable,
    C::ReferencedTable: BuildableTable,
    Self: TryMaySetColumns<<C as HorizontalSameAsKey>::HostColumns>,
    TableBuilder<<C as SingletonForeignKey>::ReferencedTable>:
        MayGetColumns<<C as HorizontalSameAsKey>::ForeignColumns>,
    TableBuilderBundle<C::Table>: crate::TrySetDiscretionaryBuilder<C>,
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles:
        TypedIndex<<C::Table as AncestorOfIndex<T>>::Idx, TableBuilderBundle<C::Table>>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        let columns = builder.may_get_columns();
        self.try_may_set_columns(columns)?;
        self.bundles.map_mut(|builder_bundle| {
            builder_bundle
                .try_set_discretionary_builder(builder.clone())
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
    #[inline]
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
    #[inline]
    fn insert(&self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        self.bundles.0.insert(conn)
    }
}

// Recursive cases for tuples of size 2-32 are generated by the macro
#[diesel_builders_macros::impl_completed_table_builder_nested_insert]
mod completed_table_builder_impls {}
