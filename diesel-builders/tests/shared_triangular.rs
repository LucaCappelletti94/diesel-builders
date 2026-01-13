//! Shared code to setup triangular relation tests.
use diesel_builders::prelude::*;

// Table A models
#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = parent_table)]
/// Model for parent table.
pub struct Parent {
    /// Primary key.
    id: i32,
    /// A column in the parent table.
    parent_field: String,
}

#[derive(Queryable, Selectable, Identifiable, TableModel, PartialEq, Eq, Debug)]
#[table_model(surrogate_key)]
#[diesel(table_name = satellite_table)]
#[table_model(foreign_key(parent_id, (parent_table::id)))]
/// Model for mandatory table.
pub struct Satellite {
    /// Primary key.
    id: i32,
    /// Foreign key to parent table.
    parent_id: i32,
    /// A column in the satellite table.
    field: String,
    /// Another column in the satellite table.
    another_field: Option<String>,
}

// Define table index that can be referenced by foreign keys
unique_index!(satellite_table::id, satellite_table::field);
unique_index!(satellite_table::id, satellite_table::another_field);
unique_index!(satellite_table::id, satellite_table::parent_id);

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
        "CREATE TABLE satellite_table (
            id INTEGER PRIMARY KEY NOT NULL,
            parent_id INTEGER NOT NULL REFERENCES parent_table(id),
            field TEXT NOT NULL,
            another_field TEXT,
			UNIQUE(id, field),
            UNIQUE(id, another_field),
			UNIQUE(id, parent_id)
        )",
    )
    .execute(conn)?;

    Ok(())
}
