//! Tests for nullable foreign key relationships.
mod shared;
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = parent_table)]
/// Model for parent table.
pub struct Parent {
    /// Primary key.
    id: i32,
}

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(surrogate_key)]
/// Model for child table with nullable foreign key to parent.
pub struct Child {
    /// Primary key.
    id: i32,
    /// Nullable foreign key to parent.
    parent_id: Option<i32>,
}

#[test]
fn test_nullable_foreign_key_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query("CREATE TABLE parent_table (id INTEGER PRIMARY KEY NOT NULL)")
        .execute(&mut conn)?;
    diesel::sql_query("CREATE TABLE child_table (id INTEGER PRIMARY KEY NOT NULL, parent_id INTEGER REFERENCES parent_table(id))").execute(&mut conn)?;

    let child = child_table::table::builder().parent_id(None).insert(&mut conn)?;

    // This should fail with NotFound because parent_id is None
    let result = child.foreign::<(child_table::parent_id,), (parent_table::id,)>(&mut conn);

    assert!(matches!(result, Err(diesel::result::Error::NotFound)));

    Ok(())
}
