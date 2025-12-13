//! Submodule for the completed table builder and related impls.

use std::ops::Sub;

use crate::builder_bundle::RecursiveBundleInsert;
use crate::{
    BuilderError, GetNestedColumns, HasNestedTables, HasTableExt, Insert, NestedTables,
    TrySetHomogeneousNestedColumns, TrySetHomogeneousNestedColumnsCollection, Typed,
    TypedNestedTuple, ValidateColumn, VerticalSameAsGroup,
};
use diesel::Table;
use diesel::associations::HasTable;
use tuplities::prelude::{
    FlattenNestedTuple, NestTuple, NestedTupleIndex, NestedTupleIndexMut, NestedTupleTryFrom,
};

use crate::{
    AncestorOfIndex, BuildableTable, BuilderResult, BundlableTable, CompletedTableBuilderBundle,
    DescendantOf, IncompleteBuilderError, TableBuilder, TableExt, TrySetColumn, TypedColumn,
};

/// A completed builder for creating insertable models for a Diesel table and
/// its ancestors.
pub struct RecursiveTableBuilder<T: diesel::Table, Depth, NestedBundles> {
    /// The insertable models for the table and its ancestors.
    nested_bundles: NestedBundles,
    /// Marker for the table and depth.
    _markers: std::marker::PhantomData<(T, Depth)>,
}

impl<T: diesel::Table, Depth, NestedBundles> RecursiveTableBuilder<T, Depth, NestedBundles> {
    /// Create a new `RecursiveTableBuilder` from the provided nested builder bundles.
    ///
    /// This is a private convenience constructor used during `TryFrom` conversions
    /// when assembling a builder from its nested parts.
    fn from_nested_bundles(nested_bundles: NestedBundles) -> Self {
        RecursiveTableBuilder {
            nested_bundles,
            _markers: std::marker::PhantomData,
        }
    }
}

/// Trait defining the insertion of a builder into the database.
pub trait RecursiveBuilderInsert<Error, Conn>: HasTableExt {
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
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error>;
}

impl<T, Error, Conn> RecursiveBuilderInsert<Error, Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    T::NestedAncestorBuilders: NestTuple,
    RecursiveTableBuilder<T, typenum::U0, T::NestedCompletedAncestorBuilders>:
        TryFrom<Self, Error = IncompleteBuilderError>
            + RecursiveBuilderInsert<Error, Conn>
            + HasTable<Table = T>,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error> {
        let completed_builder: RecursiveTableBuilder<
            T,
            typenum::U0,
            T::NestedCompletedAncestorBuilders,
        > = self.try_into()?;
        completed_builder.recursive_insert(conn)
    }
}

impl<T: BuildableTable, Conn> Insert<Conn> for TableBuilder<T>
where
    Self: RecursiveBuilderInsert<<Self::Table as TableExt>::Error, Conn>,
{
    #[inline]
    fn insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, <Self::Table as TableExt>::Error> {
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

impl<T, C, Depth, Bundles> ValidateColumn<C> for RecursiveTableBuilder<T, Depth, Bundles>
where
    Bundles: NestedTupleIndex<
            <<C::Table as AncestorOfIndex<T>>::Idx as Sub<Depth>>::Output,
            Element = CompletedTableBuilderBundle<C::Table>,
        >,
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T, Idx: Sub<Depth>> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: ValidateColumn<C>,
    TableBuilder<T>: ValidateColumn<C>,
{
    type Error = <CompletedTableBuilderBundle<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn validate_column_in_context(&self, value: &<C as Typed>::Type) -> Result<(), Self::Error> {
        self.nested_bundles
            .nested_index()
            .validate_column_in_context(value)
    }
}

impl<T, C, Depth, Bundles> TrySetColumn<C> for RecursiveTableBuilder<T, Depth, Bundles>
where
    Bundles: NestedTupleIndexMut<
            <<C::Table as AncestorOfIndex<T>>::Idx as Sub<Depth>>::Output,
            Element = CompletedTableBuilderBundle<C::Table>,
        >,
    T: BuildableTable + DescendantOf<C::Table>,
    C: VerticalSameAsGroup,
    C::Table: AncestorOfIndex<T, Idx: Sub<Depth>> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: TrySetColumn<C>,
    TableBuilder<T>: TrySetColumn<C>,
{
    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.try_set_homogeneous_nested_columns(value.clone())?;
        self.nested_bundles
            .nested_index_mut()
            .try_set_column(value)?;
        Ok(self)
    }
}

// Base case: single element nested tuple
impl<T: diesel::Table, Depth, Error, Conn, Head> RecursiveBuilderInsert<Error, Conn>
    for RecursiveTableBuilder<T, Depth, (Head,)>
where
    Conn: diesel::connection::LoadConnection,
    Head: RecursiveBundleInsert<Error, Conn>,
    Self: HasTableExt<Table = Head::Table>,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error> {
        self.nested_bundles.0.recursive_bundle_insert(conn)
    }
}

// Recursive case: nested 2-tuple (Head, Tail) where Tail is itself a nested tuple
impl<T, Depth, Error, Conn, Head, Tail> RecursiveBuilderInsert<Error, Conn>
    for RecursiveTableBuilder<T, Depth, (Head, Tail)>
where
    T: TableExt,
    Conn: diesel::connection::LoadConnection,
    Head: RecursiveBundleInsert<Error, Conn> + HasTable,
    Tail: FlattenNestedTuple + HasNestedTables,
    <Head::Table as TableExt>::Model:
        GetNestedColumns<<Head::Table as TableExt>::NestedPrimaryKeyColumns>,
    // Tail: HasNestedTables (moved into the combined bound above)
    Depth: core::ops::Add<typenum::U1>,
    RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail>:
        RecursiveBuilderInsert<Error, Conn, Table = T>
            + TrySetHomogeneousNestedColumnsCollection<
                Error,
                <<Head::Table as TableExt>::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleType,
                <Tail::NestedTables as NestedTables>::NestedPrimaryKeyColumnsCollection,
            >,
{
    #[inline]
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error> {
        // Insert the first table and get its model (with primary keys)
        let first = self.nested_bundles.0;
        let model: <Head::Table as TableExt>::Model =
            first.recursive_bundle_insert(conn)?;
        // Extract primary keys and set them in the tail builder
        let mut tail_builder = RecursiveTableBuilder::from_nested_bundles(self.nested_bundles.1);
        tail_builder
            .try_set_homogeneous_nested_columns_collection(model.get_nested_columns())
            .map_err(BuilderError::Validation)?;
        // Recursively insert the tail
        tail_builder.recursive_insert(conn)
    }
}

impl<T> TryFrom<TableBuilder<T>>
    for RecursiveTableBuilder<T, typenum::U0, T::NestedCompletedAncestorBuilders>
where
    T: BuildableTable,
{
    type Error = IncompleteBuilderError;

    #[inline]
    fn try_from(value: TableBuilder<T>) -> Result<Self, Self::Error> {
        Ok(RecursiveTableBuilder::from_nested_bundles(
            <T::NestedCompletedAncestorBuilders as NestedTupleTryFrom<
                T::NestedAncestorBuilders,
                IncompleteBuilderError,
            >>::nested_tuple_try_from(value.bundles)?,
        ))
    }
}
