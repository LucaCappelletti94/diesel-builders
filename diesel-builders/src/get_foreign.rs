//! Submodule defining the `GetForeign` trait for Diesel table models.

use tuplities::prelude::NestTuple;
use tuplities::prelude::NestedTupleFrom;

use crate::TableIndex;
use crate::TypedNestedTuple;
use crate::columns::NonEmptyNestedProjection;
use crate::columns::NonEmptyProjection;
use crate::load_query_builder::LoadFirst;
use crate::load_query_builder::LoadMany;
use crate::load_query_builder::LoadManySorted;
use crate::{GetNestedColumns, TableExt};

/// The `GetForeign` trait allows retrieving the foreign table
/// model curresponding to specified foreign columns from a host table model.
pub trait GetForeign<
    Conn,
    HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
    ForeignColumns: TableIndex<Table: TableExt>,
>: GetNestedColumns<HostColumns::Nested>
{
    /// Retrieve the foreign table model corresponding to the specified
    /// foreign columns from the host table model.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn first_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>;

    /// Retrieve all foreign table model corresponding to the specified
    /// foreign columns from the host table model.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn many_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<ForeignColumns::Table as TableExt>::Model>>;

    /// Retrieve all foreign table model corresponding to the specified
    /// foreign columns from the host table model, sorted by the foreign columns' order.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn sorted_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<ForeignColumns::Table as TableExt>::Model>>;
}

impl<Conn, HostColumns, ForeignColumns, T> GetForeign<Conn, HostColumns, ForeignColumns> for T
where
    T: GetNestedColumns<HostColumns::Nested>,
    HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
    ForeignColumns: TableIndex<
            Table: TableExt,
            Nested: NonEmptyNestedProjection<
                Table = <ForeignColumns as NonEmptyProjection>::Table,
            > + LoadFirst<Conn>
                        + LoadMany<Conn>
                        + LoadManySorted<Conn>,
        >,
    <ForeignColumns::Nested as TypedNestedTuple>::NestedTupleType:
        NestedTupleFrom<<HostColumns::Nested as TypedNestedTuple>::NestedTupleType>,
{
    fn first_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<<ForeignColumns>::Table as TableExt>::Model> {
        let host_values: <<HostColumns as NestTuple>::Nested as TypedNestedTuple>::NestedTupleType =
            self.get_nested_columns();
        <ForeignColumns::Nested as LoadFirst<Conn>>::load_first(host_values, conn)
    }

    fn many_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<<ForeignColumns>::Table as TableExt>::Model>> {
        let host_values = self.get_nested_columns();
        <ForeignColumns::Nested as LoadMany<Conn>>::load_many(host_values, conn)
    }

    fn sorted_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<<ForeignColumns>::Table as TableExt>::Model>> {
        let host_values = self.get_nested_columns();
        <ForeignColumns::Nested as LoadManySorted<Conn>>::load_many_sorted(host_values, conn)
    }
}

/// Helper trait to execute foreign key queries with the column generics
/// at the method instead of at the trait-level like in [`GetForeign`].
pub trait GetForeignExt<Conn> {
    /// Returns the first foreign object associated to the provided foreign key.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn first_foreign<HostColumns, ForeignColumns>(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>
    where
        Self: GetForeign<Conn, HostColumns, ForeignColumns>,
        HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
        ForeignColumns: TableIndex<Table: TableExt>,
    {
        <Self as GetForeign<Conn, HostColumns, ForeignColumns>>::first_foreign(self, conn)
    }

    /// Returns all foreign objects associated to the provided foreign key.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn many_foreign<HostColumns, ForeignColumns>(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<ForeignColumns::Table as TableExt>::Model>>
    where
        Self: GetForeign<Conn, HostColumns, ForeignColumns>,
        HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
        ForeignColumns: TableIndex<Table: TableExt>,
    {
        <Self as GetForeign<Conn, HostColumns, ForeignColumns>>::many_foreign(self, conn)
    }

    /// Returns all foreign objects associated to the provided foreign key, sorted by the foreign columns.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn sorted_foreign<HostColumns, ForeignColumns>(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<ForeignColumns::Table as TableExt>::Model>>
    where
        Self: GetForeign<Conn, HostColumns, ForeignColumns>,
        HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
        ForeignColumns: TableIndex<Table: TableExt>,
    {
        <Self as GetForeign<Conn, HostColumns, ForeignColumns>>::sorted_foreign(self, conn)
    }
}

impl<T, Conn> GetForeignExt<Conn> for T {}
