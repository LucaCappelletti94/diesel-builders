//! Submodule defining the `Insert` trait, which executes the insertion of a
//! builder into the database, following the dependencies between tables.

use diesel_additions::{HasTableAddition, TableAddition};

/// Trait defining the insertion of a builder into the database.
pub trait NestedInsert<Conn: diesel::connection::LoadConnection>: HasTableAddition {
    /// Insert the builder's data into the database using the provided
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    fn nested_insert(
        self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<Self::Table as TableAddition>::Model>;
}
