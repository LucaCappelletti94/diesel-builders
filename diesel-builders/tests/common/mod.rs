//! Common utilities for tests.

use diesel::{prelude::*, sqlite::SqliteConnection};

/// Establish a SQLite connection with all necessary PRAGMAs enabled.
///
/// This function creates an in-memory SQLite database connection and enables
/// important PRAGMAs for testing:
/// - `foreign_keys = ON`: Enforces foreign key constraints
/// - `recursive_triggers = ON`: Allows triggers to be recursive
/// - `journal_mode = WAL`: Uses Write-Ahead Logging for better concurrency
pub fn establish_test_connection() -> Result<SqliteConnection, diesel::ConnectionError> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Enable foreign key constraints
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .expect("Failed to enable foreign keys");

    // Enable recursive triggers
    diesel::sql_query("PRAGMA recursive_triggers = ON")
        .execute(&mut conn)
        .expect("Failed to enable recursive triggers");

    // Set journal mode to WAL for better performance
    diesel::sql_query("PRAGMA journal_mode = WAL")
        .execute(&mut conn)
        .expect("Failed to set journal mode");

    Ok(conn)
}
