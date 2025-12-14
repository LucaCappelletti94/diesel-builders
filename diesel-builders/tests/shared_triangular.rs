//! Shared code to setup triangular relation tests.
use diesel_builders::prelude::*;

// Table A models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = parent_table)]
/// Model for parent table.
pub struct Parent {
    /// Primary key.
    id: i32,
    /// A column in the parent table.
    parent_field: String,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = mandatory_table)]
/// Model for mandatory table.
pub struct Mandatory {
    /// Primary key.
    id: i32,
    /// Foreign key to parent table.
    parent_id: i32,
    /// A column in the mandatory table.
    mandatory_field: String,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = discretionary_table)]
/// Model for discretionary table.
pub struct Discretionary {
    /// Primary key.
    id: i32,
    /// Foreign key to parent table.
    parent_id: i32,
    /// A column in the discretionary table.
    discretionary_field: String,
}

// Define table index that can be referenced by foreign keys
index!(mandatory_table::id, mandatory_table::mandatory_field);
index!(mandatory_table::id, mandatory_table::parent_id);
index!(
    discretionary_table::id,
    discretionary_table::discretionary_field
);

/// Setups the triangular relation tables in the given connection.
///
/// # Errors
///
/// Returns an `Err` if any of the SQL DDL statements fail.
pub fn setup_triangular_tables(
    conn: &mut diesel::SqliteConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::RunQueryDsl;

    diesel::sql_query(
        "CREATE TABLE parent_table (
            id INTEGER PRIMARY KEY NOT NULL,
            parent_field TEXT NOT NULL
        )",
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE TABLE mandatory_table (
            id INTEGER PRIMARY KEY NOT NULL,
            parent_id INTEGER NOT NULL REFERENCES parent_table(id),
            mandatory_field TEXT,
			UNIQUE(id, mandatory_field),
			UNIQUE(id, parent_id)
        )",
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE TABLE discretionary_table (
            id INTEGER PRIMARY KEY NOT NULL,
            parent_id INTEGER NOT NULL REFERENCES parent_table(id),
            discretionary_field TEXT,
			UNIQUE(id, discretionary_field)
        )",
    )
    .execute(conn)?;

    Ok(())
}
