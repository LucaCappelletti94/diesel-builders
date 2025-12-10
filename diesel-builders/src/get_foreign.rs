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

use crate::Columns;
use crate::ForeignKey;
use crate::TableIndex;
use crate::TypedNestedTuple;
use crate::TypedTuple;
use crate::columns::NestedColumns;
use crate::columns::NonEmptyProjection;
use crate::{GetNestedColumns, TableExt};

/// The `GetForeign` trait allows retrieving the foreign table
/// model curresponding to specified foreign columns from a host table model.
pub trait GetForeign<
    Conn,
    ForeignColumns: TableIndex,
    HostColumns: Columns<TupleType = <ForeignColumns as TypedTuple>::TupleType, Nested: NestedColumns>,
>: GetNestedColumns<<HostColumns as NestTuple>::Nested> where
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
    ) -> diesel::QueryResult<<<ForeignColumns as NonEmptyProjection>::Table as TableExt>::Model>;
}

impl<
    Conn,
    ForeignColumns: TableIndex + EqAll<<ForeignColumns as TypedTuple>::TupleType>,
    HostColumns: NonEmptyProjection<
            TupleType = <ForeignColumns as TypedTuple>::TupleType,
            Nested: TypedNestedTuple<
                NestedTupleType: FlattenNestedTuple<
                    Flattened = <ForeignColumns as TypedTuple>::TupleType,
                >,
            >,
        > + ForeignKey<ForeignColumns>,
    T: GetNestedColumns<<HostColumns as NestTuple>::Nested> + HasTable<Table = HostColumns::Table>,
> GetForeign<Conn, ForeignColumns, HostColumns> for T
where
    <ForeignColumns as NonEmptyProjection>::Table: TableExt,
    Conn: diesel::connection::LoadConnection,
    <ForeignColumns as NonEmptyProjection>::Table:
        SelectDsl<<<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns>,
    <<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output:
        FilterDsl<<ForeignColumns as EqAll<<ForeignColumns as TypedTuple>::TupleType>>::Output>,
    <<<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<<ForeignColumns as TypedTuple>::TupleType>>::Output,
    >>::Output: LimitDsl + RunQueryDsl<Conn>,
    for<'query> <<<<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<<ForeignColumns as TypedTuple>::TupleType>>::Output,
    >>::Output as LimitDsl>::Output:
        LoadQuery<'query, Conn, <<ForeignColumns as NonEmptyProjection>::Table as TableExt>::Model>,
{
    fn get_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<<ForeignColumns as NonEmptyProjection>::Table as TableExt>::Model>
    {
        let foreign_table: <ForeignColumns as NonEmptyProjection>::Table = Default::default();
        let foreign_columns = <ForeignColumns as NestTuple>::Nested::default().flatten();
        let foreign_key_values = self.get_nested_columns().flatten();
        RunQueryDsl::first(
            FilterDsl::filter(
                SelectDsl::select(
                    foreign_table,
                    <<ForeignColumns as NonEmptyProjection>::Table as Table>::all_columns(),
                ),
                foreign_columns.eq_all(foreign_key_values),
            ),
            conn,
        )
    }
}
