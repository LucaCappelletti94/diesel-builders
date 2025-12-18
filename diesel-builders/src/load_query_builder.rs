//! Module providing a helper trait to construct a load query to be further specialized
//! and completed by other traits.

use diesel::Table;
use diesel::expression_methods::EqAll;
use diesel::query_dsl::methods::FilterDsl;
use diesel::query_dsl::methods::LimitDsl;
use diesel::query_dsl::methods::LoadQuery;
use diesel::query_dsl::methods::OffsetDsl;
use diesel::query_dsl::methods::OrderDsl;
use diesel::query_dsl::methods::SelectDsl;
use tuplities::prelude::{FlattenNestedTuple, NestedTupleInto};

use crate::{
    TableExt,
    columns::{NonEmptyNestedProjection, TupleToOrder},
};

/// The `LoadQueryBuilder` trait allows retrieving the foreign table
/// model curresponding to specified foreign columns from a host table model.
pub trait LoadQueryBuilder: NonEmptyNestedProjection {
    /// The type of the constructed load query.
    type LoadQuery;

    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - A nested tuple of values corresponding to the foreign columns.
    fn load_query(values: impl NestedTupleInto<Self::NestedTupleValueType>) -> Self::LoadQuery;
}

impl<NestedColumns> LoadQueryBuilder for NestedColumns
where
    NestedColumns: NonEmptyNestedProjection,
    NestedColumns::Flattened:
        EqAll<<NestedColumns::NestedTupleValueType as FlattenNestedTuple>::Flattened>,
    NestedColumns::Table: TableExt + SelectDsl<<NestedColumns::Table as Table>::AllColumns>,
    <NestedColumns::Table as SelectDsl<<NestedColumns::Table as Table>::AllColumns>>::Output:
        FilterDsl<
            <NestedColumns::Flattened as EqAll<
                <NestedColumns::NestedTupleValueType as FlattenNestedTuple>::Flattened,
            >>::Output,
        >,
{
    type LoadQuery = <<NestedColumns::Table as SelectDsl<
        <NestedColumns::Table as Table>::AllColumns,
    >>::Output as FilterDsl<
        <NestedColumns::Flattened as EqAll<
            <NestedColumns::NestedTupleValueType as FlattenNestedTuple>::Flattened,
        >>::Output,
    >>::Output;

    fn load_query(values: impl NestedTupleInto<Self::NestedTupleValueType>) -> Self::LoadQuery {
        let table: NestedColumns::Table = Default::default();
        let columns = NestedColumns::default().flatten();
        let values: NestedColumns::NestedTupleValueType = values.nested_tuple_into();
        FilterDsl::filter(
            SelectDsl::select(table, <NestedColumns::Table as Table>::all_columns()),
            columns.eq_all(values.flatten()),
        )
    }
}

/// The `LoadFirst` trait allows retrieving the first record from a load query.
pub trait LoadFirst<Conn>: LoadQueryBuilder<Table: TableExt> {
    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - A nested tuple of values corresponding to the foreign columns.
    /// * `conn` - A mutable reference to the Diesel connection to use for the query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn load_first(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<Self::Table as TableExt>::Model>;
}

impl<Conn, NestedColumns> LoadFirst<Conn> for NestedColumns
where
    Conn: diesel::connection::LoadConnection,
    NestedColumns: LoadQueryBuilder + NonEmptyNestedProjection<Table: TableExt>,
    NestedColumns::LoadQuery: LimitDsl + diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> <Self::LoadQuery as LimitDsl>::Output:
        LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn load_first(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<Self::Table as TableExt>::Model> {
        let query = Self::load_query(values).limit(1);
        diesel::query_dsl::RunQueryDsl::get_result::<<Self::Table as TableExt>::Model>(query, conn)
    }
}

/// The `LoadMany` trait allows retrieving several records from a load query.
pub trait LoadMany<Conn>: LoadQueryBuilder<Table: TableExt> {
    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - A nested tuple of values corresponding to the foreign columns.
    /// * `conn` - A mutable reference to the Diesel connection to use for the query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails.
    fn load_many(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<Self::Table as TableExt>::Model>>;
}

impl<Conn, NestedColumns> LoadMany<Conn> for NestedColumns
where
    Conn: diesel::connection::LoadConnection,
    NestedColumns: LoadQueryBuilder + NonEmptyNestedProjection<Table: TableExt>,
    NestedColumns::LoadQuery: diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> Self::LoadQuery: LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn load_many(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<Self::Table as TableExt>::Model>> {
        let query = Self::load_query(values);
        diesel::query_dsl::RunQueryDsl::load::<<Self::Table as TableExt>::Model>(query, conn)
    }
}

/// The `LoadManySorted` trait allows retrieving several records from a load query, sorted by a given expression.
pub trait LoadManySorted<Conn>: LoadQueryBuilder<Table: TableExt> {
    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - The values to filter the load query by.
    /// * `conn` - A mutable reference to the Diesel connection to use for the query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn load_many_sorted(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<Self::Table as TableExt>::Model>>;
}

impl<Conn, NestedColumns> LoadManySorted<Conn> for NestedColumns
where
    Conn: diesel::connection::LoadConnection,
    NestedColumns: LoadQueryBuilder + NonEmptyNestedProjection<Table: TableExt>,
    <NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns: TupleToOrder,
    NestedColumns::LoadQuery: OrderDsl<
            <<NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
        > + diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> <Self::LoadQuery as OrderDsl<
        <<NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output: LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn load_many_sorted(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<Self::Table as TableExt>::Model>> {
        let order =
            <NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns::default().to_order();
        let query = Self::load_query(values).order(order);
        diesel::query_dsl::RunQueryDsl::load::<<Self::Table as TableExt>::Model>(query, conn)
    }
}

/// The `LoadManySortedPaginated` trait allows retrieving several records from a load query,
/// sorted by a given expression with offset and limit for pagination.
pub trait LoadManySortedPaginated<Conn>: LoadQueryBuilder<Table: TableExt> {
    /// Constructs a paginated load query.
    ///
    /// # Arguments
    ///
    /// * `values` - The values to filter the load query by.
    /// * `offset` - The number of records to skip.
    /// * `limit` - The maximum number of records to return.
    /// * `conn` - A mutable reference to the Diesel connection to use for the query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails.
    fn load_many_sorted_paginated(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        offset: i64,
        limit: i64,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<Self::Table as TableExt>::Model>>;
}

impl<Conn, NestedColumns> LoadManySortedPaginated<Conn> for NestedColumns
where
    Conn: diesel::connection::LoadConnection,
    NestedColumns: LoadQueryBuilder + NonEmptyNestedProjection<Table: TableExt>,
    <NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns: TupleToOrder,
    NestedColumns::LoadQuery: OrderDsl<
            <<NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
        > + diesel::query_dsl::RunQueryDsl<Conn>,
    <NestedColumns::LoadQuery as OrderDsl<
        <<NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output: LimitDsl + OffsetDsl,
    <<NestedColumns::LoadQuery as OrderDsl<
        <<NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output as LimitDsl>::Output: OffsetDsl,
    for<'query> <<<NestedColumns::LoadQuery as OrderDsl<
        <<NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output as LimitDsl>::Output as OffsetDsl>::Output:
        LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn load_many_sorted_paginated(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        offset: i64,
        limit: i64,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<<Self::Table as TableExt>::Model>> {
        let order =
            <NestedColumns::Table as TableExt>::NestedPrimaryKeyColumns::default().to_order();
        let query = Self::load_query(values)
            .order(order)
            .limit(limit)
            .offset(offset);
        diesel::query_dsl::RunQueryDsl::load::<<Self::Table as TableExt>::Model>(query, conn)
    }
}
