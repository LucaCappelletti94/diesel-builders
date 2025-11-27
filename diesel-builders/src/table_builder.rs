//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use std::marker::PhantomData;

use diesel::{Table, associations::HasTable};
use diesel_additions::{
    DefaultTuple, MayGetColumn, MayGetInsertableTableModelColumn, SetColumn,
    SetInsertableTableModelColumn, TableAddition, Tables, TrySetColumn,
    TrySetInsertableTableModelColumn, TrySetInsertableTableModelHomogeneousColumn, TypedColumn,
};
use diesel_relations::vertical_same_as_group::VerticalSameAsGroup;
use typed_tuple::prelude::{TupleIndex0, TypedTuple};

use crate::{
    BuildableColumn, BuildableTables, BuilderBundles, CompletedTableBuilderBundle, NestedInsert,
    TrySetMandatoryBuilder, buildable_table::BuildableTable,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    bundles: <T::AncestorsWithSelf as BuildableTables>::BuilderBundles,
}

impl<T: BuildableTable> Default for TableBuilder<T> {
    fn default() -> Self {
        Self { bundles: DefaultTuple::default_tuple() }
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
    for CompletedTableBuilder<T, <T::AncestorsWithSelf as BuildableTables>::CompletedBuilderBundles>
where
    T: BuildableTable,
{
    type Error = anyhow::Error;

    fn try_from(
        value: TableBuilder<T>,
    ) -> Result<
        CompletedTableBuilder<
            T,
            <T::AncestorsWithSelf as BuildableTables>::CompletedBuilderBundles,
        >,
        Self::Error,
    > {
        let bundles = value.bundles.try_complete()?;
        Ok(CompletedTableBuilder { bundles, table: PhantomData })
    }
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: TypedColumn,
    <T::AncestorsWithSelf as Tables>::InsertableModels: MayGetInsertableTableModelColumn<C>,
{
    fn maybe_get(&self) -> Option<&<C as diesel_additions::TypedColumn>::Type> {
        todo!()
    }
}

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: VerticalSameAsGroup + TypedColumn,
    <T::AncestorsWithSelf as Tables>::InsertableModels: SetInsertableTableModelColumn<C>,
{
    fn set(&mut self, value: &<C as TypedColumn>::Type) {
        todo!()
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: VerticalSameAsGroup + TypedColumn,
    <T::AncestorsWithSelf as Tables>::InsertableModels: TrySetInsertableTableModelColumn<C>,
    <T::AncestorsWithSelf as Tables>::InsertableModels:
        TrySetInsertableTableModelHomogeneousColumn<C::VerticalSameAsColumns>,
{
    fn try_set(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<()> {
        todo!()
    }
}

impl<C, T> TrySetMandatoryBuilder<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: BuildableColumn,
    Self: MayGetColumn<C>,
{
    fn try_set(
        &mut self,
        _builder: TableBuilder<<C as diesel::Column>::Table>,
    ) -> anyhow::Result<()> {
        // if self.maybe_get().is_some() {
        //     anyhow::bail!(
        //         "Column {} was already set in insertable models for table {}.",
        //         C::NAME,
        //         core::any::type_name::<T>(),
        //     );
        // }
        // if self.associated_builders.maybe_get().is_some() {
        //     anyhow::bail!(
        //         "Associated builder for column {} was already set in table {}.",
        //         C::NAME,
        //         core::any::type_name::<T>(),
        //     );
        // }

        // self.associated_builders.set(builder);
        // Ok(())
        todo!()
    }
}

impl<Conn, T> NestedInsert<Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilder<T, <T::AncestorsWithSelf as BuildableTables>::CompletedBuilderBundles>:
        NestedInsert<Conn, Table = T>,
{
    fn nested_insert(
        self,
        conn: &mut Conn,
    ) -> anyhow::Result<<Self::Table as TableAddition>::Model> {
        let completed_builder: CompletedTableBuilder<
            T,
            <T::AncestorsWithSelf as BuildableTables>::CompletedBuilderBundles,
        > = self.try_into()?;
        completed_builder.nested_insert(conn)
    }
}

impl<Conn, T> NestedInsert<Conn> for CompletedTableBuilder<T, (CompletedTableBuilderBundle<T>,)>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilderBundle<T>: NestedInsert<Conn, Table = T>,
{
    fn nested_insert(self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        Ok(self.bundles.0.nested_insert(conn)?)
    }
}

impl<Conn, T, T1> NestedInsert<Conn>
    for CompletedTableBuilder<T, (CompletedTableBuilderBundle<T1>, CompletedTableBuilderBundle<T>)>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    T1: BuildableTable,
    CompletedTableBuilderBundle<T1>: NestedInsert<Conn, Table = T1>,
    (CompletedTableBuilderBundle<T1>, CompletedTableBuilderBundle<T>): TypedTuple<
            TupleIndex0,
            CompletedTableBuilderBundle<T1>,
            SplitLeftInclusive = (CompletedTableBuilderBundle<T1>,),
            SplitRightExclusive = (CompletedTableBuilderBundle<T>,),
        >,
    CompletedTableBuilder<T, (CompletedTableBuilderBundle<T>,)>: NestedInsert<Conn, Table = T>,
{
    fn nested_insert(self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        let ((first,), bundles) = self.bundles.split_left();
        let model: T1::Model = first.nested_insert(conn)?;
        // TODO: execute updates relative to extensions and triangular same-as here.
        let next_builder: CompletedTableBuilder<T, (CompletedTableBuilderBundle<T>,)> =
            CompletedTableBuilder { bundles, table: PhantomData };
        next_builder.nested_insert(conn)
    }
}
