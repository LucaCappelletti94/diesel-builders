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
    fn nested_all_columns() -> Self::NestedAllColumns;

    /// Constructs an inner join query.
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
