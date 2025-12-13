//! Submodule defining the `Descendant` trait.

use diesel::associations::HasTable;
use diesel::expression_methods::EqAll;
use diesel::query_dsl::methods::{FilterDsl, LimitDsl, LoadQuery, SelectDsl};
use diesel::{RunQueryDsl, Table};
use tuplities::prelude::{FlattenNestedTuple, NestTuple, NestedTuplePopFront, NestedTuplePushBack};
use typenum::Unsigned;

use crate::columns::NonEmptyNestedProjection;
use crate::{
    GetNestedColumns, NestedBundlableTables, TableExt, TableIndex, Tables, tables::NestedTables,
};

/// Marker trait for root table models (tables with no ancestors).
///
/// This trait should be derived on Model structs to automatically generate
/// the `Descendant` implementation for their associated table type.
pub trait Root: crate::TableExt {}

/// A trait marker for getting the ancestor index of a table.
pub trait AncestorOfIndex<T: DescendantOf<Self>>: TableExt + Descendant {
    /// Tuple index marker of the ancestor table in the descendant's ancestor
    /// list.
    type Idx: Unsigned;
}

/// A trait for Diesel tables that have ancestor tables.
/// This trait enforces that all tables in an inheritance hierarchy share the same
/// root ancestor (and thus the same primary key type).
pub trait DescendantOf<T: TableExt + Descendant>: Descendant<Root = T::Root> {}

impl<T> DescendantOf<T> for T where T: Descendant {}

/// A trait for a model associated to a diesel table which is descended from
/// another table.
pub trait ModelDescendantOf<Conn, T: TableExt + Descendant>:
    HasTable<Table: DescendantOf<T>>
{
    /// Returns the ancestor model associated to this descendant model.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn ancestor(&self, conn: &mut Conn) -> diesel::QueryResult<<T as TableExt>::Model>;
}

/// Helper trait to execute ancestor queries with the table generic at the method
/// instead of at the trait-level like in [`ModelDescendantOf`].
pub trait ModelDescendantExt<Conn> {
    /// Returns the ancestor model associated to this descendant model.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn ancestor<M>(&self, conn: &mut Conn) -> diesel::QueryResult<M>
    where
        M: HasTable<Table: TableExt<Model = M> + Descendant>,
        Self: ModelDescendantOf<Conn, M::Table>,
    {
        <Self as ModelDescendantOf<Conn, M::Table>>::ancestor(self, conn)
    }
}

impl<M, Conn> ModelDescendantExt<Conn> for M {}

impl<Conn, T, M> ModelDescendantOf<Conn, T> for M
where
    T: TableExt + Descendant + SelectDsl<<T as Table>::AllColumns>,
    M: HasTable<Table: DescendantOf<T>>
        + GetNestedColumns<<T::NestedPrimaryKeyColumns as NestTuple>::Nested>,
    T::NestedPrimaryKeyColumns: TableIndex<
            Table = T,
            Nested: NonEmptyNestedProjection<
                NestedTupleType: FlattenNestedTuple<
                    Flattened = <T::NestedPrimaryKeyColumns as crate::TypedTuple>::TupleType,
                >,
            >,
        > + EqAll<<T::NestedPrimaryKeyColumns as crate::TypedTuple>::TupleType>,
    Conn: diesel::connection::LoadConnection,
    <T as SelectDsl<<T as Table>::AllColumns>>::Output: FilterDsl<
        <T::NestedPrimaryKeyColumns as EqAll<
            <T::NestedPrimaryKeyColumns as crate::TypedTuple>::TupleType,
        >>::Output,
    >,
    <<T as SelectDsl<<T as Table>::AllColumns>>::Output as FilterDsl<
        <T::NestedPrimaryKeyColumns as EqAll<
            <T::NestedPrimaryKeyColumns as crate::TypedTuple>::TupleType,
        >>::Output,
    >>::Output: LimitDsl + RunQueryDsl<Conn>,
    for<'query> <<<T as SelectDsl<<T as Table>::AllColumns>>::Output as FilterDsl<
        <T::NestedPrimaryKeyColumns as EqAll<
            <T::NestedPrimaryKeyColumns as crate::TypedTuple>::TupleType,
        >>::Output,
    >>::Output as LimitDsl>::Output: LoadQuery<'query, Conn, <T as TableExt>::Model>,
{
    fn ancestor(&self, conn: &mut Conn) -> diesel::QueryResult<<T as TableExt>::Model> {
        let ancestor_table: T = Default::default();
        let ancestor_pk_columns =
            <T::NestedPrimaryKeyColumns as NestTuple>::Nested::default().flatten();
        let descendant_pk_values = self.get_nested_columns().flatten();
        RunQueryDsl::first(
            FilterDsl::filter(
                SelectDsl::select(ancestor_table, <T as Table>::all_columns()),
                ancestor_pk_columns.eq_all(descendant_pk_values),
            ),
            conn,
        )
    }
}

/// A trait marker for getting the ancestor tables of a descendant table.
pub trait NestedAncestorsOf<T: Descendant<Ancestors = <Self as FlattenNestedTuple>::Flattened>>:
    NestedTables
{
}

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: TableExt {
    /// The ancestor tables of this table.
    type Ancestors: Tables<Nested: NestedAncestorsOf<Self> + NestedTuplePushBack<Self>>;
    /// The root of the ancestor hierarchy. When the current
    /// table is the root, this is itself.
    type Root: Root;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant {
    /// The ancestor tables of this table, including itself.
    type NestedAncestorsWithSelf: NestedTuplePopFront<Front = Self::Root> + NestedBundlableTables;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant,
    <T::Ancestors as NestTuple>::Nested: NestedTuplePushBack<Self>,
    <<T::Ancestors as NestTuple>::Nested as NestedTuplePushBack<Self>>::Output:
        NestedBundlableTables + NestedTuplePopFront<Front = T::Root>,
{
    type NestedAncestorsWithSelf =
        <<T::Ancestors as NestTuple>::Nested as NestedTuplePushBack<Self>>::Output;
}

impl<T> NestedAncestorsOf<T> for () where T: Descendant<Ancestors = ()> {}

impl<T, A> NestedAncestorsOf<T> for (A,)
where
    A: AncestorOfIndex<T>,
    T: Descendant<Ancestors = (A,)> + DescendantOf<A> + diesel::query_source::TableNotEqual<A>,
{
}

impl<T, Head, Tail> NestedAncestorsOf<T> for (Head, Tail)
where
    (Head, Tail): NestedTables,
    Head: AncestorOfIndex<T>,
    T: Descendant<Ancestors = <(Head, Tail) as FlattenNestedTuple>::Flattened>
        + DescendantOf<Head>
        + diesel::query_source::TableNotEqual<Head>,
{
}
