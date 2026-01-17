//! Reproduction case for generic insert type mismatch.

use diesel::associations::HasTable;
use diesel_builders::{BuilderResult, DescendantWithSelf, NestedTables, TableBuilder, prelude::*};

/// Generic insert function using `TableBuilder`
///
/// # Arguments
///
/// * `builder` - A `TableBuilder` instance for the table to insert into.
/// * `conn` - A mutable reference to the database connection.
///
/// # Errors
///
/// Returns an error if the insertion fails.
pub fn simplest_compilable_insert<T, C>(
    builder: TableBuilder<T>,
    conn: &mut C,
) -> BuilderResult<T::Model, T::Error>
where
    T: BuildableTable,
    TableBuilder<T>: Insert<C> + HasTable<Table = T>,
{
    builder.insert(conn)
}

/// Generic nested insert function using `TableBuilder`
///
/// # Arguments
///
/// * `builder` - A `TableBuilder` instance for the table to insert into.
/// * `conn` - A mutable reference to the database connection.
///
/// # Errors
///
/// Returns an error if the nested insertion fails.
pub fn simplest_compilable_insert_nested<T, C>(
    builder: TableBuilder<T>,
    conn: &mut C,
) -> BuilderResult<
    <<T as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    T::Error,
>
where
    T: BuildableTable,
    TableBuilder<T>: Insert<C> + HasTable<Table = T>,
{
    builder.insert_nested(conn)
}

#[test]
fn test_simplest_compilable_insert() {
    // nothing to run, just need to compile
}
