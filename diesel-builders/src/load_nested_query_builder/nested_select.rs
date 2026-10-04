//! Submodule providing a `NestedSelect` trait which constructs a select
//! query for an n-uple of nested tables.

use diesel::{
    Expression, QueryDsl, Table, dsl::Select, query_dsl::methods::SelectDsl,
    query_source::QueryRelation,
};

use crate::NestedTables;

/// The `NestedSelect` trait allows constructing a select query
/// for an n-uple of nested tables.
pub trait NestedSelect<NT>: Sized + SelectDsl<Self::NestedAllColumns> {
    /// The `AllColumns` nested tuple type.
    type NestedAllColumns: Expression;
    /// The type of the constructed select.
    type NestedSelect;

    /// Returns an instance of `AllColumns` nested tuple.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel::query_dsl::methods::SelectDsl;
    /// use diesel_builders::load_nested_query_builder::{NestedInnerJoin, NestedSelect};
    /// use schema::*;
    ///
    /// let mut conn = connection_with_data()?;
    /// type Join = <profiles::table as NestedInnerJoin>::JoinQuery;
    /// let join = <profiles::table as NestedInnerJoin>::nested_inner_join();
    /// let columns = <Join as NestedSelect<(users::table, (profiles::table,))>>::nested_all_columns();
    /// let select = SelectDsl::select(join, columns);
    /// let rows: Vec<(User, (Profile,))> = select.load(&mut conn)?;
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
    ///         .eq([(1, "Ada", 20, Some("Ace"), 1, "Ada", 3)])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn nested_all_columns() -> Self::NestedAllColumns;

    /// Constructs the select query over the n-uple of nested tables.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::load_nested_query_builder::{NestedInnerJoin, NestedSelect};
    /// use schema::*;
    ///
    /// let mut conn = connection_with_data()?;
    /// type Join = <profiles::table as NestedInnerJoin>::JoinQuery;
    /// let join = <profiles::table as NestedInnerJoin>::nested_inner_join();
    /// let select = <Join as NestedSelect<(users::table, (profiles::table,))>>::nested_select(join);
    /// let rows: Vec<(User, (Profile,))> = select.load(&mut conn)?;
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
    ///         .eq([(1, "Ada", 20, Some("Ace"), 1, "Ada", 3)])
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn nested_select(self) -> Self::NestedSelect;
}

impl<Head, Q> NestedSelect<(Head,)> for Q
where
    Head: Table,
    Q: Sized + SelectDsl<(<Head as Table>::AllColumns,)>,
    (<Head as Table>::AllColumns,): Expression,
{
    type NestedAllColumns = (<Head as Table>::AllColumns,);
    type NestedSelect = Select<Q, Self::NestedAllColumns>;

    fn nested_all_columns() -> Self::NestedAllColumns {
        (<Head as Table>::all_columns(),)
    }
    fn nested_select(self) -> Self::NestedSelect {
        SelectDsl::select(self, <Self as NestedSelect<(Head,)>>::nested_all_columns())
    }
}

impl<Head, Tail, Q> NestedSelect<(Head, Tail)> for Q
where
    Head: QueryRelation,
    Tail: NestedTables,
    Q: Sized
        + QueryDsl
        + NestedSelect<Tail>
        + SelectDsl<(
            <Head as QueryRelation>::AllColumns,
            <Q as NestedSelect<Tail>>::NestedAllColumns,
        )>,
    (<Head as QueryRelation>::AllColumns, <Q as NestedSelect<Tail>>::NestedAllColumns): Expression,
{
    type NestedAllColumns =
        (<Head as QueryRelation>::AllColumns, <Q as NestedSelect<Tail>>::NestedAllColumns);

    type NestedSelect = Select<Q, Self::NestedAllColumns>;

    fn nested_all_columns() -> Self::NestedAllColumns {
        (<Head as QueryRelation>::all_columns(), <Q as NestedSelect<Tail>>::nested_all_columns())
    }

    fn nested_select(self) -> Self::NestedSelect {
        SelectDsl::select(self, <Self as NestedSelect<(Head, Tail)>>::nested_all_columns())
    }
}
