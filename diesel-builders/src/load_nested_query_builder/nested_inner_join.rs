//! Submodule providing a `NestedInnerJoin` trait which constructs an inner join
//! query for an n-uple of nested tables.

use diesel::{QueryDsl, dsl::InnerJoin, query_dsl::JoinWithImplicitOnClause, query_source::Inner};

use crate::{NestedTables, TableExt};

/// The `NestedInnerJoin` trait allows constructing an inner join query
/// for an n-uple of nested tables.
pub trait NestedInnerJoin: NestedTables {
    /// The type of the constructed join query.
    type JoinQuery;

    /// Constructs an inner join query.
    fn nested_inner_join() -> Self::JoinQuery;
}

impl<Head> NestedInnerJoin for (Head,)
where
    Head: TableExt,
{
    type JoinQuery = Head;

    fn nested_inner_join() -> Self::JoinQuery {
        Default::default()
    }
}

impl<Head, Tail> NestedInnerJoin for (Head, Tail)
where
    Head: TableExt,
    Tail:
        NestedInnerJoin<JoinQuery: JoinWithImplicitOnClause<Head, Inner> + QueryDsl> + NestedTables,
    (Head, Tail): NestedTables,
{
    type JoinQuery = InnerJoin<<Tail as NestedInnerJoin>::JoinQuery, Head>;

    fn nested_inner_join() -> Self::JoinQuery {
        let tail_join = Tail::nested_inner_join();
        tail_join.inner_join(<Head as Default>::default())
    }
}
