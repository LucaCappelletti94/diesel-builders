//! Module providing a helper trait to construct a load query to be further
//! specialized and completed by other traits.

use diesel::{
    expression_methods::EqAll,
    query_dsl::methods::{FilterDsl, LimitDsl, LoadQuery, OffsetDsl, OrderDsl},
};
use tuplities::prelude::{FlattenNestedTuple, NestedTupleInto};

use crate::{
    DescendantWithSelf, NestedColumns, NestedTables, TableExt, ancestors::DescendantOfAll,
    columns::TupleToOrder,
};
mod nested_inner_join;
pub use nested_inner_join::NestedInnerJoin;
mod nested_select;
pub use nested_select::NestedSelect;

/// The `LoadNestedQueryBuilder` trait allows retrieving the leaf table
/// model and all of its ancestor models corresponding to the provided columns.
pub trait LoadNestedQueryBuilder<
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
>: NestedColumns
{
    /// The type of the constructed load query.
    type LoadQuery;

    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - A nested tuple of values corresponding to the foreign
    ///   columns.
    fn load_nested_query(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
    ) -> Self::LoadQuery;
}

impl<NCS, LeafTable> LoadNestedQueryBuilder<LeafTable> for NCS
where
    LeafTable: DescendantWithSelf<
            NestedAncestorsWithSelf: NestedInnerJoin<
                JoinQuery: NestedSelect<LeafTable::NestedAncestorsWithSelf>,
            >,
        > + DescendantOfAll<Self::NestedTables>,
    NCS: NestedColumns,
    NCS::Flattened: EqAll<<NCS::NestedTupleValueType as FlattenNestedTuple>::Flattened>,
    <<LeafTable::NestedAncestorsWithSelf as NestedInnerJoin>::JoinQuery as NestedSelect<
        LeafTable::NestedAncestorsWithSelf,
    >>::NestedSelect:
        FilterDsl<
            <NCS::Flattened as EqAll<
                <NCS::NestedTupleValueType as FlattenNestedTuple>::Flattened,
            >>::Output,
        >,
{
    type LoadQuery =
        <<<LeafTable::NestedAncestorsWithSelf as NestedInnerJoin>::JoinQuery as NestedSelect<
            LeafTable::NestedAncestorsWithSelf,
        >>::NestedSelect as FilterDsl<
            <NCS::Flattened as EqAll<
                <NCS::NestedTupleValueType as FlattenNestedTuple>::Flattened,
            >>::Output,
        >>::Output;

    fn load_nested_query(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
    ) -> Self::LoadQuery {
        let inner_join = LeafTable::NestedAncestorsWithSelf::nested_inner_join();
        let columns = NCS::default().flatten();
        let values: NCS::NestedTupleValueType = values.nested_tuple_into();
        FilterDsl::filter(inner_join.nested_select(), columns.eq_all(values.flatten()))
    }
}

/// The `LoadNestedFirst` trait allows retrieving the first record set from a
/// nested load query.
pub trait LoadNestedFirst<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Returns the first record set matching the load query.
    ///
    /// # Arguments
    ///
    /// * `values` - A nested tuple of values corresponding to the foreign
    ///   columns.
    /// * `conn` - A mutable reference to the Diesel connection to use for the
    ///   query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error if the
    ///   query fails or if no matching record is found.
    fn load_nested_first(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    >;
}

impl<NCS, LeafTable, Conn> LoadNestedFirst<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    NCS::LoadQuery: LimitDsl + diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> <NCS::LoadQuery as LimitDsl>::Output: LoadQuery<
        'query,
        Conn,
        <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    >,
{
    fn load_nested_first(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    > {
        let query = Self::load_nested_query(values).limit(1);
        diesel::query_dsl::RunQueryDsl::get_result::<
            <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
        >(query, conn)
    }
}

/// The `LoadNestedMany` trait allows retrieving several record sets from a
/// nested load query.
pub trait LoadNestedMany<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - A nested tuple of values corresponding to the foreign
    ///   columns.
    /// * `conn` - A mutable reference to the Diesel connection to use for the
    ///   query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error if the
    ///   query fails.
    fn load_nested_many(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        Vec<<<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels>,
    >;
}

impl<NCS, LeafTable, Conn> LoadNestedMany<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    NCS::LoadQuery: diesel::query_dsl::RunQueryDsl<Conn>
        + for<'query> LoadQuery<
            'query,
            Conn,
            <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
        >,
{
    fn load_nested_many(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        Vec<<<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels>,
    > {
        let query = Self::load_nested_query(values);
        diesel::query_dsl::RunQueryDsl::load::<
            <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
        >(query, conn)
    }
}

/// The `LoadNestedSorted` trait allows retrieving several record sets from
/// a nested load query, sorted by the leaf table's primary key.
pub trait LoadNestedSorted<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Constructs a load query.
    ///
    /// # Arguments
    ///
    /// * `values` - The values to filter the load query by.
    /// * `conn` - A mutable reference to the Diesel connection to use for the
    ///   query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error if the
    ///   query fails or if no matching record is found.
    fn load_nested_sorted(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        Vec<<<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels>,
    >;
}

impl<NCS, LeafTable, Conn> LoadNestedSorted<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    <LeafTable as TableExt>::NestedPrimaryKeyColumns: TupleToOrder,
    NCS::LoadQuery: OrderDsl<
            <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
        > + diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> <NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output: LoadQuery<
        'query,
        Conn,
        <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    >,
{
    fn load_nested_sorted(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        Vec<<<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels>,
    > {
        let order = <LeafTable as TableExt>::NestedPrimaryKeyColumns::default().to_order();
        let query = Self::load_nested_query(values).order(order);
        diesel::query_dsl::RunQueryDsl::load::<
            <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
        >(query, conn)
    }
}

/// The `LoadNestedPaginated` trait allows retrieving several records
/// from a nested load query, sorted by the leaf table's primary key with
/// offset and limit for pagination.
pub trait LoadNestedPaginated<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Constructs a paginated load query.
    ///
    /// # Arguments
    ///
    /// * `values` - The values to filter the load query by.
    /// * `offset` - The number of records to skip.
    /// * `limit` - The maximum number of records to return.
    /// * `conn` - A mutable reference to the Diesel connection to use for the
    ///   query
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error if the
    ///   query fails.
    fn load_nested_paginated(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        offset: i64,
        limit: i64,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        Vec<<<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels>,
    >;
}

impl<NCS, LeafTable, Conn> LoadNestedPaginated<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    <LeafTable as TableExt>::NestedPrimaryKeyColumns: TupleToOrder,
    NCS::LoadQuery: OrderDsl<
            <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
        > + diesel::query_dsl::RunQueryDsl<Conn>,
    <NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output: LimitDsl + OffsetDsl,
    <<NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output as LimitDsl>::Output: OffsetDsl,
    for<'query> <<<NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output as LimitDsl>::Output as OffsetDsl>::Output: LoadQuery<
        'query,
        Conn,
        <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    >,
{
    fn load_nested_paginated(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        offset: i64,
        limit: i64,
        conn: &mut Conn,
    ) -> diesel::QueryResult<
        Vec<<<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels>,
    > {
        let order = <LeafTable as TableExt>::NestedPrimaryKeyColumns::default().to_order();
        let query = Self::load_nested_query(values)
            .order(order)
            .limit(limit)
            .offset(offset);
        diesel::query_dsl::RunQueryDsl::load::<
            <<LeafTable as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
        >(query, conn)
    }
}
