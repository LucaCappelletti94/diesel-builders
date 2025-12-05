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
use crate::{Columns, GetColumns, TableAddition, columns::NonEmptyProjection};

/// The `GetForeign` trait allows retrieving the foreign table
/// model curresponding to specified foreign columns from a host table model.
pub trait GetForeign<
    Conn,
    ForeignColumns: TableIndex,
    HostColumns: NonEmptyProjection<Types = <ForeignColumns as Columns>::Types>,
>: GetColumns<HostColumns> + HasTable<Table = HostColumns::Table>
{
    /// Retrieve the foreign table model corresponding to the specified
    /// foreign columns from the host table model.
    fn get_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<<ForeignColumns as NonEmptyProjection>::Table as TableAddition>::Model>;
}

impl<
    Conn,
    ForeignColumns: TableIndex,
    HostColumns: NonEmptyProjection<Types = <ForeignColumns as Columns>::Types>,
    T: GetColumns<HostColumns> + HasTable<Table=HostColumns::Table>,
> GetForeign<Conn, ForeignColumns, HostColumns> for T
where
    HostColumns: ForeignKey<ForeignColumns>,
    Conn: diesel::connection::LoadConnection,
    for<'a> ForeignColumns: EqAll<<<ForeignColumns as Columns>::Types as TupleRef>::Ref<'a>>,
    for<'a> <<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output: FilterDsl<
        <ForeignColumns as EqAll<
            <<ForeignColumns as Columns>::Types as TupleRef>::Ref<'a>,
        >>::Output,
    >,
    for<'a> <<<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<
            <<ForeignColumns as Columns>::Types as TupleRef>::Ref<'a>,
        >>::Output,
    >>::Output: LimitDsl + RunQueryDsl<Conn>,
    for<'query> <<<<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<
            <<ForeignColumns as Columns>::Types as TupleRef>::Ref<'query>,
        >>::Output,
    >>::Output as LimitDsl>::Output: LoadQuery<
            'query,
            Conn,
            <<ForeignColumns as NonEmptyProjection>::Table as TableAddition>::Model,
        >,
{
    fn get_foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<<ForeignColumns as NonEmptyProjection>::Table as TableAddition>::Model> {
        let foreign_table: <ForeignColumns as NonEmptyProjection>::Table = Default::default();
        let foreign_key_values = self.get_columns();
        RunQueryDsl::first(
            FilterDsl::filter(
                SelectDsl::select(
                    foreign_table,
                    <<ForeignColumns as NonEmptyProjection>::Table as Table>::all_columns(),
                ),
                ForeignColumns::tuple_default().eq_all(foreign_key_values),
            ),
            conn,
        )
    }
}
