//! Submodule defining the `GetForeign` trait for Diesel table models.

use diesel::RunQueryDsl;
use diesel::Table;
use diesel::associations::HasTable;
use diesel::expression_methods::EqAll;
use diesel::query_dsl::methods::FilterDsl;
use diesel::query_dsl::methods::LimitDsl;
use diesel::query_dsl::methods::LoadQuery;
use diesel::query_dsl::methods::SelectDsl;
use tuplities::prelude::*;

use crate::ForeignKey;
use crate::TableIndex;
use crate::TypedNestedTuple;
use crate::columns::NonEmptyNestedProjection;
use crate::columns::NonEmptyProjection;
use crate::{GetNestedColumns, TableExt};

/// The `GetForeign` trait allows retrieving the foreign table
/// model curresponding to specified foreign columns from a host table model.
pub trait GetForeign<
    Conn,
    HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
    ForeignColumns: TableIndex,
>: GetNestedColumns<HostColumns::Nested> where
    ForeignColumns::Table: TableExt,
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
    fn get_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>;
}

impl<
    Conn,
    ForeignColumns,
    HostColumns,
    T,
> GetForeign<Conn, HostColumns, ForeignColumns> for T
where
    T: GetNestedColumns<<HostColumns as NestTuple>::Nested> + HasTable<Table = HostColumns::Table>,
    HostColumns: ForeignKey<ForeignColumns> + NonEmptyProjection<Nested: NonEmptyNestedProjection>,
    ForeignColumns: TableIndex<
        Nested: TypedNestedTuple<
            NestedTupleType: NestedTupleFrom<<HostColumns::Nested as TypedNestedTuple>::NestedTupleType>,
        >,
    > + EqAll<<<ForeignColumns::Nested as TypedNestedTuple>::NestedTupleType as FlattenNestedTuple>::Flattened>,
    ForeignColumns::Table:
        TableExt + SelectDsl<<ForeignColumns::Table as Table>::AllColumns>,
    Conn: diesel::connection::LoadConnection,
    <ForeignColumns::Table as SelectDsl<
        <ForeignColumns::Table as Table>::AllColumns,
    >>::Output:
        FilterDsl<<ForeignColumns as EqAll<<<ForeignColumns::Nested as TypedNestedTuple>::NestedTupleType as FlattenNestedTuple>::Flattened>>::Output>,
    <<ForeignColumns::Table as SelectDsl<
        <ForeignColumns::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<<<ForeignColumns::Nested as TypedNestedTuple>::NestedTupleType as FlattenNestedTuple>::Flattened>>::Output,
    >>::Output: LimitDsl + RunQueryDsl<Conn>,
    for<'query> <<<ForeignColumns::Table as SelectDsl<
        <ForeignColumns::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<<<ForeignColumns::Nested as TypedNestedTuple>::NestedTupleType as FlattenNestedTuple>::Flattened>>::Output,
    >>::Output as LimitDsl>::Output:
        LoadQuery<'query, Conn, <ForeignColumns::Table as TableExt>::Model>,
{
    fn get_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>
    {
        let foreign_table: ForeignColumns::Table = Default::default();
        let foreign_columns = <ForeignColumns as NestTuple>::Nested::default().flatten();
        let host_values = self.get_nested_columns();
        let foreign_key_values: <ForeignColumns::Nested as TypedNestedTuple>::NestedTupleType = host_values.nested_tuple_into();
        RunQueryDsl::first(
            FilterDsl::filter(
                SelectDsl::select(
                    foreign_table,
                    <ForeignColumns::Table as Table>::all_columns(),
                ),
                foreign_columns.eq_all(foreign_key_values.flatten()),
            ),
            conn,
        )
    }
}

/// Helper trait to execute foreign key queries with the column generics
/// at the method instead of at the trait-level like in [`GetForeign`].
pub trait GetForeignExt<Conn> {
    /// Returns the foreign object associated to the provided foreign key.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn get_foreign<HostColumns, ForeignColumns>(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>
    where
        Self: GetForeign<Conn, HostColumns, ForeignColumns>,
        HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
        ForeignColumns: TableIndex<Table: TableExt>,
    {
        <Self as GetForeign<Conn, HostColumns, ForeignColumns>>::get_foreign(self, conn)
    }
}

impl<T, Conn> GetForeignExt<Conn> for T {}
