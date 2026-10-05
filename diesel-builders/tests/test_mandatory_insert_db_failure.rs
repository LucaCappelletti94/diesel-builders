//! A database-level constraint violation on a mandatory associated insert,
//! not a Rust-level validation rejection, still rolls back the whole
//! transaction and surfaces as `BuilderError::Diesel`.

mod shared;

use diesel::prelude::*;
use diesel_builders::{BuilderError, RecursiveBuilderInsert, prelude::*};

diesel::allow_tables_to_appear_in_same_query!(db_failure_parents, db_failure_codes);
diesel::allow_tables_to_appear_in_same_query!(db_failure_parents, db_failure_children);
diesel::allow_tables_to_appear_in_same_query!(db_failure_codes, db_failure_children);

#[derive(Queryable, Clone, Selectable, Identifiable, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = db_failure_parents)]
/// An ancestor table unrelated to the constraint under test.
pub struct Parent {
    /// Primary key.
    id: i32,
    /// A column present only so the table is non-empty.
    #[table_model(default = 0)]
    marker: i32,
}

#[derive(Queryable, Clone, Selectable, Identifiable, TableModel)]
#[table_model(surrogate_key, foreign_key(parent_id, (db_failure_parents::id)))]
#[diesel(table_name = db_failure_codes)]
/// The mandatory associated table, carrying a uniqueness constraint that
/// only the database can enforce.
pub struct Code {
    /// Primary key.
    id: i32,
    /// Foreign key to the parent table.
    parent_id: i32,
    /// A globally unique value, not validated for uniqueness in Rust.
    value: String,
}

unique_index!(db_failure_codes::id, db_failure_codes::parent_id);

#[derive(Queryable, Clone, Selectable, Identifiable, TableModel)]
#[table_model(ancestors = db_failure_parents)]
#[diesel(table_name = db_failure_children)]
/// A child carrying a mandatory reference to `Code`.
pub struct Child {
    #[same_as(db_failure_codes::parent_id)]
    /// Primary key, shared with the parent and propagated to `Code`.
    id: i32,
    #[mandatory(db_failure_codes)]
    /// Foreign key to the mandatory `Code` row.
    mandatory_id: i32,
}

fn setup() -> Result<diesel::SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE db_failure_parents (id INTEGER PRIMARY KEY, marker INTEGER NOT NULL)",
    )
    .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE db_failure_codes (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER NOT NULL REFERENCES db_failure_parents(id),
            value TEXT NOT NULL UNIQUE,
            UNIQUE(id, parent_id)
        )",
    )
    .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE db_failure_children (
            id INTEGER PRIMARY KEY REFERENCES db_failure_parents(id),
            mandatory_id INTEGER NOT NULL REFERENCES db_failure_codes(id)
        )",
    )
    .execute(&mut conn)?;
    Ok(conn)
}

#[test]
fn unique_violation_on_mandatory_insert_rolls_back_the_whole_transaction()
-> Result<(), Box<dyn std::error::Error>> {
    let mut conn = setup()?;

    // A value row taken by an unrelated, already-committed parent.
    let existing_parent = db_failure_parents::table::builder().insert(&mut conn)?;
    db_failure_codes::table::builder()
        .parent_id(existing_parent.id)
        .value("taken")
        .insert(&mut conn)?;

    // Inserting a brand new child whose mandatory Code reuses that value
    // must fail at the database, not at Rust-level validation: nothing in
    // this schema has a Rust validator for `value` at all.
    let attempt = db_failure_children::table::builder()
        .mandatory(db_failure_codes::table::builder().value("taken"))
        .recursive_insert(&mut conn);

    let Err(error) = attempt else {
        return Err("expected the unique value to be rejected by the database".into());
    };
    assert!(
        matches!(
            error,
            BuilderError::Diesel(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ))
        ),
        "expected a unique-violation database error, got {error:?}"
    );

    // The whole transaction rolled back: no new parent row was left behind
    // by the aborted insert, and the pre-existing rows are untouched.
    let parent_count: i64 = db_failure_parents::table.count().get_result(&mut conn)?;
    assert_eq!(parent_count, 1);
    let code_count: i64 = db_failure_codes::table.count().get_result(&mut conn)?;
    assert_eq!(code_count, 1);
    let child_count: i64 = db_failure_children::table.count().get_result(&mut conn)?;
    assert_eq!(child_count, 0);

    Ok(())
}
