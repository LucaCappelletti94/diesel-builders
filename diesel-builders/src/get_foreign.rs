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
use crate::Typed;
use crate::columns::NonEmptyProjection;
use crate::{GetColumns, TableAddition};

/// The `GetForeign` trait allows retrieving the foreign table
/// model curresponding to specified foreign columns from a host table model.
pub trait GetForeign<
    Conn,
    ForeignColumns: TableIndex,
    HostColumns: Columns<Type = <ForeignColumns as Typed>::Type>,
>: GetColumns<HostColumns>
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
    ) -> diesel::QueryResult<<<ForeignColumns as NonEmptyProjection>::Table as TableAddition>::Model>;
}

impl<
    Conn,
    ForeignColumns: TableIndex,
    HostColumns: NonEmptyProjection<Type = <ForeignColumns as Typed>::Type>,
    T: GetColumns<HostColumns> + HasTable<Table=HostColumns::Table>,
> GetForeign<Conn, ForeignColumns, HostColumns> for T
where
    HostColumns: ForeignKey<ForeignColumns>,
    Conn: diesel::connection::LoadConnection,
    <ForeignColumns as NonEmptyProjection>::Table: SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >,
    for<'a> ForeignColumns: EqAll<<<ForeignColumns as Typed>::Type as TupleRef>::Ref<'a>>,
    for<'a> <<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output: FilterDsl<
        <ForeignColumns as EqAll<
            <<ForeignColumns as Typed>::Type as TupleRef>::Ref<'a>,
        >>::Output,
    >,
    for<'a> <<<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<
            <<ForeignColumns as Typed>::Type as TupleRef>::Ref<'a>,
        >>::Output,
    >>::Output: LimitDsl + RunQueryDsl<Conn>,
    for<'query> <<<<ForeignColumns as NonEmptyProjection>::Table as SelectDsl<
        <<ForeignColumns as NonEmptyProjection>::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <ForeignColumns as EqAll<
            <<ForeignColumns as Typed>::Type as TupleRef>::Ref<'query>,
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
        let foreign_key_values = self.get_columns_ref();
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
