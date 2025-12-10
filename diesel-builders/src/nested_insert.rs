//! Submodule defining the `Insert` trait, which executes the insertion of a
//! builder into the database, following the dependencies between tables.

use diesel::associations::HasTable;

use crate::{BuilderResult, HasTableExt, InsertableTableModel, TableExt};

/// Trait defining the insertion of a builder into the database.
pub trait Insert<Conn>: HasTableExt {
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
    ) -> BuilderResult<
        <<Self as HasTable>::Table as TableExt>::Model,
        <<<Self as HasTable>::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    >;
}
