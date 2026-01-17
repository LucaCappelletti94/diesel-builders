//! Submodule defining the `Insert` trait, which executes the insertion of a
//! builder into the database, following the dependencies between tables.

use crate::{BuilderResult, DescendantWithSelf, HasTableExt, NestedTables, TableExt};

/// Trait defining the insertion of a builder into the database.
pub trait Insert<Conn>: HasTableExt<Table: DescendantWithSelf> {
    /// Insert the builder's data into the database using the provided
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails or if any database constraints
    /// are violated.
    fn insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, <Self::Table as TableExt>::Error>;

    /// Insert the builder's data into the database using the provided
    /// connection, returning a nested tuple with all of the inserted models.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails or if any database constraints
    /// are violated.
    fn insert_nested(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self::Table as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels, <Self::Table as TableExt>::Error>;
}
