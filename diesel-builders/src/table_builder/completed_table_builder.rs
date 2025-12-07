//! Submodule for the completed table builder and related impls.

use std::ops::Sub;

use crate::builder_bundle::RecursiveBundleInsert;
use crate::get_set_columns::TrySetHomogeneous;
use crate::tables::HasTables;
use crate::{GetColumns, HasTableAddition, InsertableTableModel, Typed};
use diesel::Table;
use diesel::associations::HasTable;
use tuplities::prelude::{
    FlattenNestedTuple, NestTuple, NestedTupleIndexMut, TupleRef, TupleTryFrom,
};

use crate::{
    AncestorOfIndex, BuildableTable, BuilderResult, BundlableTable, BundlableTables,
    CompletedTableBuilderBundle, DescendantOf, IncompleteBuilderError, TableAddition, TableBuilder,
    Tables, TrySetColumn, TypedColumn,
};

/// A completed builder for creating insertable models for a Diesel table and
/// its ancestors.
struct RecursiveTableBuilder<T, Depth, Bundles> {
    /// The insertable models for the table and its ancestors.
    bundles: Bundles,
    /// Marker for the table and depth.
    _markers: std::marker::PhantomData<(T, Depth)>,
}

impl<T, Depth, Bundles> RecursiveTableBuilder<T, Depth, Bundles> {
    fn from_bundles(bundles: Bundles) -> Self {
        RecursiveTableBuilder {
            bundles,
            _markers: std::marker::PhantomData,
        }
    }
}

/// Trait defining the insertion of a builder into the database.
pub trait RecursiveBuilderInsert<Error, Conn>: HasTableAddition {
    /// Insert the builder's data into the database using the provided
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails or if any database constraints
    /// are violated.
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error>;
}

impl<T, Error, Conn> RecursiveBuilderInsert<Error, Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles: NestTuple,
    RecursiveTableBuilder<
        T,
        typenum::U0,
        <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles,
    >: TryFrom<Self, Error = crate::IncompleteBuilderError>,
    RecursiveTableBuilder<
        T,
        typenum::U0,
        <<T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles as NestTuple>::Nested,
    >: RecursiveBuilderInsert<Error, Conn> + HasTable<Table = T>,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error> {
        use NestTuple;
        let completed_builder: RecursiveTableBuilder<
            T,
            typenum::U0,
            <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles,
        > = self.try_into()?;
        // Convert flat tuple to nested tuple for recursive processing
        let nested = completed_builder.bundles.nest();
        let nested_builder = RecursiveTableBuilder::from_bundles(nested);
        nested_builder.recursive_insert(conn)
    }
}

impl<T: BuildableTable, Conn> crate::Insert<Conn> for TableBuilder<T>
where
    Self: RecursiveBuilderInsert<<<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error, Conn>
{
    #[inline]
    fn insert(self, conn: &mut Conn) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>{
        self.recursive_insert(conn)
    }
}

impl<T: Table + Default, Depth, Bundles> HasTable for RecursiveTableBuilder<T, Depth, Bundles> {
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T> TryFrom<TableBuilder<T>>
    for RecursiveTableBuilder<
        T,
        typenum::U0,
        <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles,
    >
where
    T: BuildableTable,
    <T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles: TupleTryFrom<
            <T::AncestorsWithSelf as BundlableTables>::BuilderBundles,
            crate::IncompleteBuilderError,
        >,
{
    type Error = IncompleteBuilderError;

    #[inline]
    fn try_from(value: TableBuilder<T>) -> Result<Self, Self::Error> {
        Ok(RecursiveTableBuilder::from_bundles(
            <<T::AncestorsWithSelf as BundlableTables>::CompletedBuilderBundles as TupleTryFrom<
                _,
                _,
            >>::tuple_try_from(value.bundles)?,
        ))
    }
}

impl<'a, T, C, Depth, Bundles> TrySetColumn<C> for RecursiveTableBuilder<T, Depth, Bundles>
where
    Self: 'a,
    Bundles: NestedTupleIndexMut<
            <<C::Table as AncestorOfIndex<T>>::Idx as Sub<Depth>>::Output,
            Element = CompletedTableBuilderBundle<C::Table>,
        >,
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T, Idx: Sub<Depth>> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: TrySetColumn<C>,
    TableBuilder<T>: TrySetColumn<C>,
{
    type Error = <CompletedTableBuilderBundle<C::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.bundles.nested_index_mut().try_set_column(value)?;
        // TODO: set vertical same-as columns in associated builders here.
        Ok(self)
    }
}

// Base case: single element nested tuple
impl<T, Depth, Error, Conn, Head> RecursiveBuilderInsert<Error, Conn>
    for RecursiveTableBuilder<T, Depth, (Head,)>
where
    Conn: diesel::connection::LoadConnection,
    Head: RecursiveBundleInsert<Error, Conn>,
    Self: HasTableAddition<Table = <Head as HasTable>::Table>,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error> {
        self.bundles.0.recursive_bundle_insert(conn)
    }
}

// Recursive case: nested 2-tuple (Head, Tail) where Tail is itself a nested tuple
impl<T, Depth, Error, Conn, Head, Tail> RecursiveBuilderInsert<Error, Conn>
    for RecursiveTableBuilder<T, Depth, (Head, Tail)>
where
    T: TableAddition,
    Conn: diesel::connection::LoadConnection,
    Head: RecursiveBundleInsert<Error, Conn> + HasTable,
    Tail: FlattenNestedTuple,
    <<Head as HasTable>::Table as TableAddition>::Model:
        GetColumns<<<Head as HasTable>::Table as TableAddition>::PrimaryKeyColumns>,
    Tail::Flattened: HasTables,
    Depth: core::ops::Add<typenum::U1>,
    RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail>: RecursiveBuilderInsert<Error, Conn, Table=T>
        + for<'a> crate::TrySetHomogeneous<
            Error,
            <<<<Head as HasTable>::Table as TableAddition>::PrimaryKeyColumns as Typed>::Type as TupleRef>::Ref<'a>,
            <<Tail::Flattened as HasTables>::Tables as Tables>::PrimaryKeyColumnsCollection,
        >,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error> {
        // Insert the first table and get its model (with primary keys)
        let first = self.bundles.0;
        let model: <<Head as HasTable>::Table as TableAddition>::Model =
            first.recursive_bundle_insert(conn)?;
        // Extract primary keys and set them in the tail builder
        let mut tail_builder = RecursiveTableBuilder::from_bundles(self.bundles.1);
        tail_builder
            .try_set_homogeneous(model.get_columns())
            .map_err(crate::BuilderError::Validation)?;
        // Recursively insert the tail
        tail_builder.recursive_insert(conn)
    }
}
