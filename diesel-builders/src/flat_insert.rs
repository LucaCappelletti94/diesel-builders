//! Submodule defining the `Insert` trait, which executes the insertion of a
//! builder into the database, following the dependencies between tables.

use diesel::RunQueryDsl;

use crate::{HasTableExt, TableExt};

/// Trait defining the insertion of a builder into the database.
pub trait FlatInsert<Conn: diesel::connection::LoadConnection>: HasTableExt {
    /// Insert the builder's data into the database using the provided
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns a `diesel::QueryResult` error if the insertion fails.
    fn flat_insert(self, conn: &mut Conn) -> diesel::QueryResult<<Self::Table as TableExt>::Model>;
}

impl<Conn, T> FlatInsert<Conn> for T
where
    Conn: diesel::connection::LoadConnection,
    T: HasTableExt + diesel::Insertable<Self::Table>,
    diesel::query_builder::InsertStatement<
        Self::Table,
        <Self as diesel::Insertable<Self::Table>>::Values,
    >: for<'query> diesel::query_dsl::LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn flat_insert(self, conn: &mut Conn) -> diesel::QueryResult<<Self::Table as TableExt>::Model> {
        diesel::insert_into(T::table())
            .values(self)
            .get_result(conn)
    }
}
