//! Transaction boundaries for public insertion operations.

use diesel::Connection;

use crate::{
    BuilderResult, HasTableExt, RecursiveBuilderInsert, RecursiveBundleInsert, TableExt,
    TypedNestedTupleCollection,
    columns::{NestedColumns, NestedColumnsCollection},
};

/// Recursive builder insertion within an existing transaction.
pub(crate) trait InsertBuilder<Conn>: HasTableExt {
    /// The models produced by the nested builders.
    type NestedModels;

    /// The validation errors that this insertion stage can still produce.
    type ValidationError;

    /// # Errors
    ///
    /// Returns the insertion, incomplete, or validation error from any nested
    /// builder.
    fn insert_builder(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Self::ValidationError>;

    /// # Errors
    ///
    /// Returns the insertion, incomplete, or validation error from any nested
    /// builder.
    fn insert_builder_nested(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<Self::NestedModels, Self::ValidationError>;
}

/// Bundle insertion within an existing transaction.
pub(crate) trait InsertBundle<Conn>: HasTableExt {
    /// The validation errors that this insertion stage can still produce.
    type ValidationError;

    /// # Errors
    ///
    /// Returns the insertion, incomplete, or validation error from any
    /// associated record.
    fn insert_bundle(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Self::ValidationError>;
}

/// A borrowed view of stored column options, used to check completeness
/// without moving the stored values.
pub(crate) trait CheckNestedStoredOptions<H> {
    /// The parallel nested tuple of column names.
    type SameDepth;

    /// Returns the name of the first absent column, if any.
    fn first_missing_with(self, names: Self::SameDepth) -> Option<H>;
}

impl<H> CheckNestedStoredOptions<H> for () {
    type SameDepth = ();

    #[inline]
    fn first_missing_with(self, _names: ()) -> Option<H> {
        None
    }
}

impl<T, H> CheckNestedStoredOptions<H> for (&Option<T>,) {
    type SameDepth = (H,);

    #[inline]
    fn first_missing_with(self, names: (H,)) -> Option<H> {
        if self.0.is_some() { None } else { Some(names.0) }
    }
}

impl<Head, Tail, H> CheckNestedStoredOptions<H> for (&Option<Head>, Tail)
where
    Tail: CheckNestedStoredOptions<H>,
{
    type SameDepth = (H, Tail::SameDepth);

    #[inline]
    fn first_missing_with(self, names: (H, Tail::SameDepth)) -> Option<H> {
        if self.0.is_some() { self.1.first_missing_with(names.1) } else { Some(names.0) }
    }
}

/// Stores one table's generated primary key columns into a descendant
/// builder, retaining each column's exact validation error.
pub(crate) trait SetTableKeyColumns<CS: NestedColumns> {
    /// The exact validation errors of the group's columns.
    type Error;

    /// # Errors
    ///
    /// Returns the first failing column's exact validation error.
    fn set_table_key_columns(
        &mut self,
        values: CS::NestedTupleColumnType,
    ) -> Result<&mut Self, Self::Error>;
}

/// Stores several tables' generated primary key columns into a descendant
/// builder, retaining each table group's exact validation errors.
pub(crate) trait SetTableKeyColumnsCollection<NCC: NestedColumnsCollection> {
    /// The exact validation errors of the collection's table groups.
    type Error;

    /// # Errors
    ///
    /// Returns the first failing table group's exact validation errors.
    fn set_table_key_columns_collection(
        &mut self,
        values: <NCC as TypedNestedTupleCollection>::NestedCollectionType,
    ) -> Result<&mut Self, Self::Error>;
}

impl<Error, Conn, T, Models> RecursiveBuilderInsert<Error, Conn> for T
where
    Conn: Connection,
    T: InsertBuilder<Conn, ValidationError = Error, NestedModels = Models>,
{
    type NestedModels = Models;

    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error> {
        conn.transaction(|conn| self.insert_builder(conn))
    }

    fn recursive_insert_nested(self, conn: &mut Conn) -> BuilderResult<Self::NestedModels, Error> {
        conn.transaction(|conn| self.insert_builder_nested(conn))
    }
}

impl<Error, Conn, T> RecursiveBundleInsert<Error, Conn> for T
where
    Conn: Connection,
    T: InsertBundle<Conn, ValidationError = Error>,
{
    fn recursive_bundle_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error> {
        conn.transaction(|conn| self.insert_bundle(conn))
    }
}
