//! Submodule defining the `Insert` trait, which executes the insertion of a
//! builder into the database, following the dependencies between tables.

use diesel::{associations::HasTable, connection::LoadConnection};

use crate::{
    BuilderResult, HasTableAddition, InsertableTableModel, TableAddition, tables::TableModels,
};

/// Trait defining the insertion of a builder into the database.
pub trait Insert<Conn>: HasTableAddition {
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
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>;
}

/// Trait defining the insertion of a builder into the database.
pub trait RecursiveInsert<Error, Conn>: HasTableAddition {
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
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error>;
}

/// Trait defining the insertion of a tuple of builders into the database.
pub trait NestedInsertTuple<Error, Conn> {
    /// The type of the models associated with the builders in the tuple.
    type ModelsTuple: TableModels;

    /// Insert the tuple of builders' data into the database using the provided
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if any insertion fails or if any database constraints
    /// are violated.
    fn nested_insert_tuple(self, conn: &mut Conn) -> BuilderResult<Self::ModelsTuple, Error>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_nested_insert_tuple]
mod impls {}

/// Trait defining the insertion of a tuple of optional builders into the
/// database.
pub trait NestedInsertOptionTuple<Error, Conn> {
    /// The type of the optional models associated with the builders in the
    /// tuple.
    type OptionModelsTuple;

    /// Insert the tuple of optional builders' data into the database using the
    /// provided connection. If a builder is `None`, the corresponding model
    /// will also be `None`.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if any insertion fails or if any database constraints
    /// are violated.
    fn nested_insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<Self::OptionModelsTuple, Error>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_nested_insert_option_tuple]
mod option_impls {}
