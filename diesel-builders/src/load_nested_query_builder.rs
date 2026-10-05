//! Module providing a helper trait to construct a load query to be further
//! specialized and completed by other traits.

use diesel::{
    expression_methods::EqAll,
    query_dsl::methods::{FilterDsl, LimitDsl, LoadQuery, OffsetDsl, OrderDsl},
};
use tuplities::prelude::{FlattenNestedTuple, NestedTupleInto};

use crate::{
    DescendantWithSelf, NestedColumns, TableExt, ancestors::DescendantOfAll, columns::TupleToOrder,
    helper_type::NestedModel,
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

    /// Constructs an ancestor join filtered by the selected columns' nested
    /// values.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::load_nested_query_builder::LoadNestedQueryBuilder;
    /// use schema::*;
    ///
    /// let mut conn = connection_with_profiles()?;
    /// let query =
    ///     <(profiles::visits,) as LoadNestedQueryBuilder<profiles::table>>::load_nested_query((3,));
    /// let rows: Vec<(User, (Profile,))> = query.order(users::id).load(&mut conn)?;
    /// assert!(
    ///     rows.iter()
    ///         .map(|(user, (profile,))| {
    ///             (
    ///                 user.id,
    ///                 user.name.as_str(),
    ///                 user.age,
    ///                 user.nickname.as_deref(),
    ///                 profile.id,
    ///                 profile.display_name.as_str(),
    ///                 profile.visits,
    ///             )
    ///         })
    ///         .eq([
    ///             (1, "Ada", 20, Some("Ace"), 1, "Ada", 3),
    ///             (2, "Grace", 30, None, 2, "Grace", 3),
    ///             (3, "Bob", 25, None, 3, "Bob", 3),
    ///         ])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn load_nested_query(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
    ) -> Self::LoadQuery;
}

impl<NCS, LeafTable> LoadNestedQueryBuilder<LeafTable> for NCS
where
    LeafTable: DescendantWithSelf
        + NestedInnerJoin<JoinQuery: NestedSelect<LeafTable::NestedAncestorsWithSelf>>
        + DescendantOfAll<Self::NestedTables>,
    NCS: NestedColumns,
    NCS::Flattened: EqAll<<NCS::NestedTupleValueType as FlattenNestedTuple>::Flattened>,
    <<LeafTable as NestedInnerJoin>::JoinQuery as NestedSelect<
        LeafTable::NestedAncestorsWithSelf,
    >>::NestedSelect:
        FilterDsl<
            <NCS::Flattened as EqAll<
                <NCS::NestedTupleValueType as FlattenNestedTuple>::Flattened,
            >>::Output,
        >,
{
    type LoadQuery =
        <<<LeafTable as NestedInnerJoin>::JoinQuery as NestedSelect<
            LeafTable::NestedAncestorsWithSelf,
        >>::NestedSelect as FilterDsl<
            <NCS::Flattened as EqAll<
                <NCS::NestedTupleValueType as FlattenNestedTuple>::Flattened,
            >>::Output,
        >>::Output;

    fn load_nested_query(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
    ) -> Self::LoadQuery {
        let inner_join = LeafTable::nested_inner_join();
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
    /// # Errors
    ///
    /// Returns a database error if the query fails or no record matches.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection_with_data()?;
    /// let (user, (profile,)) =
    ///     <(profiles::id,) as LoadNestedFirst<profiles::table, _>>::load_nested_first(
    ///         (1,),
    ///         &mut conn,
    ///     )?;
    /// assert_eq!(user.id, profile.id);
    /// assert_eq!(user.name, profile.display_name);
    /// assert_eq!(profile, Profile { id: 1, display_name: "Ada".to_owned(), visits: 3 });
    /// assert_eq!(
    ///     user,
    ///     User { id: 1, name: "Ada".to_owned(), age: 20, nickname: Some("Ace".to_owned()) }
    /// );
    ///
    /// let missing = <(profiles::id,) as LoadNestedFirst<profiles::table, _>>::load_nested_first(
    ///     (999,),
    ///     &mut conn,
    /// );
    /// assert!(matches!(missing, Err(diesel::result::Error::NotFound)));
    /// # Ok(())
    /// # }
    /// ```
    fn load_nested_first(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<NestedModel<LeafTable>>;
}

impl<NCS, LeafTable, Conn> LoadNestedFirst<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    NCS::LoadQuery: LimitDsl + diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> <NCS::LoadQuery as LimitDsl>::Output:
        LoadQuery<'query, Conn, NestedModel<LeafTable>>,
{
    fn load_nested_first(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<NestedModel<LeafTable>> {
        let query = Self::load_nested_query(values).limit(1);
        diesel::query_dsl::RunQueryDsl::get_result::<NestedModel<LeafTable>>(query, conn)
    }
}

/// The `LoadNestedMany` trait allows retrieving several record sets from a
/// nested load query.
pub trait LoadNestedMany<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Returns all records matching the load query.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection_with_profiles()?;
    /// let mut rows = <(profiles::visits,) as LoadNestedMany<profiles::table, _>>::load_nested_many(
    ///     (3,),
    ///     &mut conn,
    /// )?;
    /// rows.sort_by_key(|(user, _)| user.id);
    /// assert!(
    ///     rows.iter()
    ///         .map(|(user, (profile,))| {
    ///             (
    ///                 user.id,
    ///                 user.name.as_str(),
    ///                 user.age,
    ///                 user.nickname.as_deref(),
    ///                 profile.id,
    ///                 profile.display_name.as_str(),
    ///                 profile.visits,
    ///             )
    ///         })
    ///         .eq([
    ///             (1, "Ada", 20, Some("Ace"), 1, "Ada", 3),
    ///             (2, "Grace", 30, None, 2, "Grace", 3),
    ///             (3, "Bob", 25, None, 3, "Bob", 3),
    ///         ])
    /// );
    ///
    /// let missing = <(profiles::visits,) as LoadNestedMany<profiles::table, _>>::load_nested_many(
    ///     (999,),
    ///     &mut conn,
    /// )?;
    /// assert!(missing.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    fn load_nested_many(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<NestedModel<LeafTable>>>;
}

impl<NCS, LeafTable, Conn> LoadNestedMany<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    NCS::LoadQuery: diesel::query_dsl::RunQueryDsl<Conn>
        + for<'query> LoadQuery<'query, Conn, NestedModel<LeafTable>>,
{
    fn load_nested_many(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<NestedModel<LeafTable>>> {
        let query = Self::load_nested_query(values);
        diesel::query_dsl::RunQueryDsl::load::<NestedModel<LeafTable>>(query, conn)
    }
}

/// The `LoadNestedSorted` trait allows retrieving several record sets from
/// a nested load query, sorted by the leaf table's primary key.
pub trait LoadNestedSorted<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Loads matching records in leaf primary-key order.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection_with_profiles()?;
    /// let rows = <(profiles::visits,) as LoadNestedSorted<profiles::table, _>>::load_nested_sorted(
    ///     (3,),
    ///     &mut conn,
    /// )?;
    /// assert_eq!(
    ///     rows.iter().map(|(user, (profile,))| (user.id, profile.id)).collect::<Vec<_>>(),
    ///     vec![(1, 1), (2, 2), (3, 3)],
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn load_nested_sorted(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<NestedModel<LeafTable>>>;
}

impl<NCS, LeafTable, Conn> LoadNestedSorted<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    <LeafTable as TableExt>::NestedPrimaryKeyColumns: TupleToOrder,
    NCS::LoadQuery: OrderDsl<<<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order>
        + diesel::query_dsl::RunQueryDsl<Conn>,
    for<'query> <NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output: LoadQuery<'query, Conn, NestedModel<LeafTable>>,
{
    fn load_nested_sorted(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<NestedModel<LeafTable>>> {
        let order = <LeafTable as TableExt>::NestedPrimaryKeyColumns::default().to_order();
        let query = Self::load_nested_query(values).order(order);
        diesel::query_dsl::RunQueryDsl::load::<NestedModel<LeafTable>>(query, conn)
    }
}

/// The `LoadNestedPaginated` trait allows retrieving several records
/// from a nested load query, sorted by the leaf table's primary key with
/// offset and limit for pagination.
pub trait LoadNestedPaginated<LeafTable, Conn>: LoadNestedQueryBuilder<LeafTable>
where
    LeafTable: DescendantWithSelf + DescendantOfAll<Self::NestedTables>,
{
    /// Loads matching records in leaf primary-key order with an offset and
    /// limit.
    ///
    /// # Errors
    ///
    /// Returns a database error if the query fails.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::load_nested_query_builder::LoadNestedPaginated;
    /// use schema::*;
    ///
    /// let mut conn = connection_with_profiles()?;
    /// let page =
    ///     <(profiles::visits,) as LoadNestedPaginated<profiles::table, _>>::load_nested_paginated(
    ///         (3,),
    ///         1,
    ///         2,
    ///         &mut conn,
    ///     )?;
    /// assert_eq!(page.iter().map(|(_, (profile,))| profile.id).collect::<Vec<_>>(), vec![2, 3],);
    ///
    /// let page =
    ///     <(profiles::visits,) as LoadNestedPaginated<profiles::table, _>>::load_nested_paginated(
    ///         (3,),
    ///         2,
    ///         5,
    ///         &mut conn,
    ///     )?;
    /// assert_eq!(page.iter().map(|(_, (profile,))| profile.id).collect::<Vec<_>>(), vec![3]);
    /// # Ok(())
    /// # }
    /// ```
    fn load_nested_paginated(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        offset: i64,
        limit: i64,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<NestedModel<LeafTable>>>;
}

impl<NCS, LeafTable, Conn> LoadNestedPaginated<LeafTable, Conn> for NCS
where
    Conn: diesel::connection::LoadConnection,
    NCS: LoadNestedQueryBuilder<LeafTable>,
    LeafTable: DescendantWithSelf + DescendantOfAll<NCS::NestedTables>,
    <LeafTable as TableExt>::NestedPrimaryKeyColumns: TupleToOrder,
    NCS::LoadQuery: OrderDsl<<<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order>
        + diesel::query_dsl::RunQueryDsl<Conn>,
    <NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output: LimitDsl + OffsetDsl,
    <<NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output as LimitDsl>::Output: OffsetDsl,
    for<'query> <<<NCS::LoadQuery as OrderDsl<
        <<LeafTable as TableExt>::NestedPrimaryKeyColumns as TupleToOrder>::Order,
    >>::Output as LimitDsl>::Output as OffsetDsl>::Output:
        LoadQuery<'query, Conn, NestedModel<LeafTable>>,
{
    fn load_nested_paginated(
        values: impl NestedTupleInto<Self::NestedTupleValueType>,
        offset: i64,
        limit: i64,
        conn: &mut Conn,
    ) -> diesel::QueryResult<Vec<NestedModel<LeafTable>>> {
        let order = <LeafTable as TableExt>::NestedPrimaryKeyColumns::default().to_order();
        let query = Self::load_nested_query(values).order(order).limit(limit).offset(offset);
        diesel::query_dsl::RunQueryDsl::load::<NestedModel<LeafTable>>(query, conn)
    }
}
