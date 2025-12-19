//! Submodule defining the `Descendant` trait.

use crate::columns::TupleEqAll;
use diesel::associations::HasTable;
use diesel::connection::LoadConnection;
use diesel::query_builder::{DeleteStatement, InsertStatement, IntoUpdateTarget};
use diesel::query_dsl::methods::{ExecuteDsl, FindDsl, LoadQuery, SetUpdateDsl};
use diesel::query_dsl::{DoUpdateDsl, OnConflictDsl};
use diesel::{AsChangeset, Identifiable, Insertable, QueryResult, RunQueryDsl, Table};
use tuplities::prelude::{FlattenNestedTuple, NestTuple, NestedTupleInto, NestedTuplePushBack};
use typenum::Unsigned;

use crate::load_query_builder::LoadFirst;
use crate::{GetNestedColumns, NestedBundlableTables, TableExt, Tables, tables::NestedTables};
use crate::{NestedColumns, TypedColumn, TypedNestedTuple};

/// Marker trait for root table models (tables with no ancestors).
///
/// This trait should be derived on Model structs to automatically generate
/// the `Descendant` implementation for their associated table type.
pub trait Root: TableExt {}

/// A trait marker for getting the ancestor index of a table.
pub trait AncestorOfIndex<T: DescendantOf<Self>>: Descendant {
    /// Tuple index marker of the ancestor table in the descendant's ancestor
    /// list.
    type Idx: Unsigned;
}

/// A trait for Diesel tables that have ancestor tables.
/// This trait enforces that all tables in an inheritance hierarchy share the same
/// root ancestor (and thus the same primary key type).
pub trait DescendantOf<T: Descendant>: Descendant<Root = T::Root> {}

impl<T> DescendantOf<T> for T where T: Descendant {}

/// A column from an ancestor table.
pub trait AncestorColumnOf<T: DescendantOf<Self::Table>>: TypedColumn<Table: Descendant> {}
impl<T, C> AncestorColumnOf<T> for C
where
    T: DescendantOf<C::Table>,
    C: TypedColumn<Table: Descendant>,
{
}

/// A collection of columns from ancestors of the provided descendant table.
pub trait AncestorColumnsOf<T> {}

impl<T, A: NestTuple> AncestorColumnsOf<T> for A where A::Nested: NestedAncestorColumnsOf<T> {}

/// A nested collection of columns from ancestors of the provided descendant table.
pub trait NestedAncestorColumnsOf<T>: TypedNestedTuple {}

impl<T> NestedAncestorColumnsOf<T> for () {}
impl<T, A> NestedAncestorColumnsOf<T> for (A,)
where
    A: AncestorColumnOf<T>,
    T: DescendantOf<A::Table>,
{
}
impl<T, CHead, CTail> NestedAncestorColumnsOf<T> for (CHead, CTail)
where
    T: DescendantOf<CHead::Table>,
    CHead: AncestorColumnOf<T>,
    CTail: NestedAncestorColumnsOf<T>,
    (CHead, CTail): NestedColumns,
{
}

/// A trait for a model associated to a diesel table which is descended from
/// another table.
pub trait ModelDescendantOf<Conn, T: Descendant>: HasTable<Table: DescendantOf<T>> {
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

    /// Deletes the root table record associated with this descendant model,
    /// which will cascade and delete all descendants including this instance.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the delete operation fails.
    fn delete(&self, conn: &mut Conn) -> diesel::QueryResult<usize>
    where
        Self: ModelDelete<Conn>,
    {
        <Self as ModelDelete<Conn>>::delete(self, conn)
    }
}

impl<M, Conn> ModelDescendantExt<Conn> for M {}

impl<Conn, T, M> ModelDescendantOf<Conn, T> for M
where
    T: Descendant,
    M: HasTable<Table: DescendantOf<T>> + GetNestedColumns<T::NestedPrimaryKeyColumns>,
    T::NestedPrimaryKeyColumns: LoadFirst<Conn>,
    <T::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleColumnType:
        NestedTupleInto<<T::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleColumnType>,
{
    fn ancestor(&self, conn: &mut Conn) -> diesel::QueryResult<<T as TableExt>::Model> {
        let descendant_pk_values = self.get_nested_columns();
        <T::NestedPrimaryKeyColumns as LoadFirst<Conn>>::load_first(descendant_pk_values, conn)
    }
}

/// A trait for finding a model by its ID.
pub trait ModelFind<Conn>: HasTable<Table: TableExt>
where
    for<'a> &'a Self: Identifiable,
{
    /// Finds a model by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to search for.
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails or if no matching record is found.
    fn find(
        id: <&Self as Identifiable>::Id,
        conn: &mut Conn,
    ) -> QueryResult<<Self::Table as TableExt>::Model>;

    /// Returns whether a model with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID to search for.
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the query fails.
    fn exists(id: <&Self as Identifiable>::Id, conn: &mut Conn) -> QueryResult<bool> {
        use diesel::OptionalExtension;
        match Self::find(id, conn).optional()? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

impl<Conn, M> ModelFind<Conn> for M
where
    M: HasTable<Table: TableExt>,
    Conn: diesel::connection::LoadConnection,
    for<'query> &'query M: Identifiable,
    M::Table: for<'query> FindDsl<<&'query M as Identifiable>::Id>,
    for<'query> <M::Table as FindDsl<<&'query M as Identifiable>::Id>>::Output:
        LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn find(
        id: <&Self as Identifiable>::Id,
        conn: &mut Conn,
    ) -> QueryResult<<Self::Table as TableExt>::Model> {
        M::Table::default().find(id).get_result(conn)
    }
}

/// A trait for deleting a model from its root table, which cascades to all descendants.
pub trait ModelDelete<Conn>: HasTable<Table: Descendant> {
    /// Deletes the root table record associated with this descendant model,
    /// which will cascade and delete all descendants including this instance.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the delete operation fails.
    fn delete(&self, conn: &mut Conn) -> diesel::QueryResult<usize>;
}

impl<Conn, M> ModelDelete<Conn> for M
where
    M: HasTable<Table: Descendant>,
    for<'query> &'query M: Identifiable,
    Conn: diesel::Connection,
    <M::Table as Descendant>::Root: for<'query> FindDsl<<&'query M as Identifiable>::Id>,
    for<'query> <<M::Table as Descendant>::Root as FindDsl<<&'query M as Identifiable>::Id>>::Output:
        IntoUpdateTarget<Table = <M::Table as Descendant>::Root>,
    for<'query> DeleteStatement<
        <M::Table as Descendant>::Root,
        <<<M::Table as Descendant>::Root as FindDsl<<&'query M as Identifiable>::Id>>::Output as
        IntoUpdateTarget>::WhereClause,
    >: ExecuteDsl<Conn>,
{
    fn delete(&self, conn: &mut Conn) -> diesel::QueryResult<usize> {
        let root_table: <M::Table as Descendant>::Root = Default::default();
        diesel::delete(root_table.find(self.id())).execute(conn)
    }
}

/// A trait for upserting (insert or update) a model.
///
/// This trait allows inserting a model or updating it if it already exists,
/// based on a conflict on the primary key.
///
pub trait ModelUpsert<Conn>: HasTable<Table: TableExt> {
    /// Upserts the model (insert or update on conflict).
    ///
    /// If a record with the same primary key exists, it is updated.
    /// Otherwise, a new record is inserted.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection.
    ///
    /// # Returns
    ///
    /// * The inserted or updated model.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error
    ///   if the upsert operation fails.
    fn upsert(&self, conn: &mut Conn) -> QueryResult<<Self::Table as TableExt>::Model>
    where
        Self: Sized;
}

impl<Conn, M> ModelUpsert<Conn> for M
where
    M: HasTable<Table: TableExt>
        + GetNestedColumns<<<M::Table as Table>::AllColumns as NestTuple>::Nested>,
    Conn: LoadConnection,
    <<M::Table as Table>::AllColumns as NestTuple>::Nested:
        TupleEqAll<EqAll: FlattenNestedTuple<Flattened: Insertable<M::Table> + AsChangeset<Target = M::Table>>>,
    for<'query> InsertStatement<
        Self::Table,
        <<<<<M::Table as Table>::AllColumns as NestTuple>::Nested as TupleEqAll>::EqAll as FlattenNestedTuple>::Flattened as Insertable<Self::Table>>::Values,
    >: OnConflictDsl<
        <M::Table as Table>::PrimaryKey,
        Output: DoUpdateDsl<Output: SetUpdateDsl<
            <<<<M::Table as Table>::AllColumns as NestTuple>::Nested as TupleEqAll>::EqAll as FlattenNestedTuple>::Flattened,
            Output: LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
        >>
    >,
{
    fn upsert(&self, conn: &mut Conn) -> QueryResult<<Self::Table as TableExt>::Model>
    where
        Self: Sized,
    {
        use diesel::Table;
        let table: M::Table = Default::default();
        let columns = <<M::Table as Table>::AllColumns as NestTuple>::Nested::default();
        let results: Vec<<Self::Table as TableExt>::Model> = diesel::insert_into(table)
            .values(columns.eq_all(self.get_nested_columns()).flatten())
            .on_conflict(table.primary_key())
            .do_update()
            .set(columns.eq_all(self.get_nested_columns()).flatten())
            .get_results(conn)?;

        if let Some(first) = results.into_iter().next() {
            Ok(first)
        } else {
            Err(diesel::result::Error::NotFound)
        }
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
    type Root: Root<NestedPrimaryKeyColumns: TypedNestedTuple<
        NestedTupleColumnType = <Self::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleColumnType,
        NestedTupleValueType = <Self::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleValueType,
    >>;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant + AncestorOfIndex<Self> {
    /// The ancestor tables of this table, including itself.
    type NestedAncestorsWithSelf: NestedBundlableTables;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant + AncestorOfIndex<Self>,
    <T::Ancestors as NestTuple>::Nested: NestedTuplePushBack<Self>,
    <<T::Ancestors as NestTuple>::Nested as NestedTuplePushBack<Self>>::Output:
        NestedBundlableTables,
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
