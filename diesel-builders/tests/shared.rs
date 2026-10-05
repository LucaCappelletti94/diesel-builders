//! Shared utilities and table definitions for tests.

/// Establish a `SQLite` connection with all necessary PRAGMAs enabled.
///
/// This function creates an in-memory `SQLite` database connection and enables
/// important PRAGMAs for testing:
/// - `foreign_keys = ON`: Enforces foreign key constraints
/// - `recursive_triggers = ON`: Allows triggers to be recursive
/// - `journal_mode = WAL`: Uses Write-Ahead Logging for better concurrency
///
/// # Errors
///
/// Returns an `Err` if creating the connection or setting PRAGMAs fails.
pub fn establish_connection() -> Result<diesel::SqliteConnection, Box<dyn std::error::Error>> {
    use diesel::{Connection, RunQueryDsl};
    let mut conn = diesel::SqliteConnection::establish(":memory:")?;

    // Enable foreign key constraints
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;

    // Enable recursive triggers
    diesel::sql_query("PRAGMA recursive_triggers = ON").execute(&mut conn)?;

    // Set journal mode to WAL for better performance
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(&mut conn)?;

    Ok(conn)
}
