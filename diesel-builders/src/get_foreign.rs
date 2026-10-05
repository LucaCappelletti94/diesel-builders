//! Submodule defining the `GetForeign` trait for Diesel table models.

use tuplities::prelude::{IntoNestedTupleOption, NestedTupleInto, NestedTupleOption};

use crate::{
    GetNestedColumns, TableExt, TypedNestedTuple, UniqueTableIndex,
    columns::{NonEmptyNestedProjection, NonEmptyProjection},
    load_query_builder::LoadFirst,
};

/// The `GetForeign` trait allows retrieving the foreign table
/// model corresponding to specified foreign columns from a host table model.
pub trait GetForeign<
    Conn,
    HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
    ForeignColumns: UniqueTableIndex<Table: TableExt>,
>: GetNestedColumns<HostColumns::Nested>
{
    /// Retrieve the foreign table model corresponding to the specified
    /// foreign columns from the host table model.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the
    ///   query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error if the
    ///   query fails or if no matching record is found.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetForeign;
    /// use schema::{Post, User, posts, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let post = Post::find(&1, &mut conn)?;
    /// let author: User =
    ///     GetForeign::<SqliteConnection, (posts::user_id,), (users::id,)>::foreign(&post, &mut conn)?;
    /// assert_eq!(author.id, 1);
    /// assert_eq!(author.name, "Ada");
    /// assert_eq!(author.nickname, Some("Ace".into()));
    /// # Ok(())
    /// # }
    /// ```
    fn foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>;
}

impl<Conn, HostColumns, ForeignColumns, T> GetForeign<Conn, HostColumns, ForeignColumns> for T
where
    T: GetNestedColumns<HostColumns::Nested>,
    HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
    ForeignColumns: UniqueTableIndex<
            Table: TableExt,
            Nested: NonEmptyNestedProjection<
                Table = <ForeignColumns as NonEmptyProjection>::Table,
            > + LoadFirst<Conn>,
        >,
    <HostColumns::Nested as TypedNestedTuple>::NestedTupleValueType:
        NestedTupleInto<<ForeignColumns::Nested as TypedNestedTuple>::NestedTupleValueType>,
{
    fn foreign(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<<ForeignColumns>::Table as TableExt>::Model> {
        let host_column_values = self.get_nested_columns();
        let optional_host_values: <<HostColumns::Nested as TypedNestedTuple>::NestedTupleValueType as IntoNestedTupleOption>::IntoOptions = host_column_values.nested_tuple_into();
        let Some(transposed_host_values) = optional_host_values.transpose() else {
            return Err(diesel::result::Error::NotFound);
        };
        <ForeignColumns::Nested as LoadFirst<Conn>>::load_first(transposed_host_values, conn)
    }
}

/// Helper trait to execute foreign key queries with the column generics
/// at the method instead of at the trait-level like in [`GetForeign`].
pub trait GetForeignExt<Conn> {
    /// Returns the first foreign object associated to the provided foreign key.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the Diesel connection to use for the
    ///   query.
    ///
    /// # Errors
    ///
    /// * Returns a `diesel::QueryResult` which may contain an error if the
    ///   query fails or if no matching record is found.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{Post, User, posts, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let post = Post::find(&3, &mut conn)?;
    /// let author: User = post.foreign::<(posts::user_id,), (users::id,)>(&mut conn)?;
    /// assert_eq!(author.id, 2);
    /// assert_eq!(author.name, "Grace");
    /// assert_eq!(author.nickname, None);
    /// # Ok(())
    /// # }
    /// ```
    fn foreign<HostColumns, ForeignColumns>(
        &self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<ForeignColumns::Table as TableExt>::Model>
    where
        Self: GetForeign<Conn, HostColumns, ForeignColumns>,
        HostColumns: NonEmptyProjection<Nested: NonEmptyNestedProjection>,
        ForeignColumns: UniqueTableIndex<Table: TableExt>,
    {
        <Self as GetForeign<Conn, HostColumns, ForeignColumns>>::foreign(self, conn)
    }
}

impl<T, Conn> GetForeignExt<Conn> for T {}
