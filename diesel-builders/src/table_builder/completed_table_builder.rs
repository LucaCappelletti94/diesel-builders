//! Submodule for the completed table builder and related impls.

use std::marker::PhantomData;

use diesel::{Table, associations::HasTable};
use tuple_set::TupleSet;
use tuplities::prelude::TupleTryFrom;

use crate::{
    AncestorOfIndex, BuildableTable, BuilderResult, BundlableTable, BundlableTables,
    CompletedTableBuilderBundle, DescendantOf, IncompleteBuilderError, Insert,
    InsertableTableModel, RecursiveInsert, TableAddition, TableBuilder, TrySetColumn, TypedColumn,
};

/// A completed builder for creating insertable models for a Diesel table and
/// its ancestors.
struct CompletedTableBuilder<T: Table, Bundles> {
    /// The insertable models for the table and its ancestors.
    bundles: Bundles,
    /// The table type.
    table: PhantomData<T>,
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
    <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles: TupleTryFrom<
            <T::AncestorsWithSelf as BundlableTables>::BuilderBundles,
            crate::IncompleteBuilderError,
        >,
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
        Ok(CompletedTableBuilder {
            bundles:
                <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles::tuple_try_from(
                    value.bundles,
                )?,
            table: PhantomData,
        })
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

impl<T, Conn> Insert<Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    CompletedTableBuilder<T, <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles>:
        RecursiveInsert<
                <<T as TableAddition>::InsertableModel as InsertableTableModel>::Error,
                Conn,
                Table = T,
            > + TryFrom<Self, Error = IncompleteBuilderError>,
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
        RecursiveInsert<Error, Conn, Table = T> + TryFrom<Self, Error = IncompleteBuilderError>,
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
