//! Test case for checking that vertical same-as dependencies
//! work correctly in the context where the exist a mandatory
//! table setting the same column value.

mod shared;

use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = parent_table)]
#[table_model(surrogate_key)]
/// A parent table model.
pub struct Parent {
    /// Primary key.
    id: i32,
    /// A field in the parent table.
    parent_field: String,
}

unique_index!(parent_table::id, parent_table::parent_field);

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = satellite_table)]
#[table_model(surrogate_key)]
/// A parent table model.
pub struct Mandatory {
    /// Primary key.
    id: i32,
    /// The parent table id.
    parent_id: i32,
    /// A field in the parent table.
    field: String,
}

unique_index!(satellite_table::id, satellite_table::field);
unique_index!(satellite_table::id, satellite_table::parent_id);

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
/// Model for a child table that inherits from `parent_table`.
pub struct Child {
    #[same_as(satellite_table::parent_id)]
    /// Primary key.
    id: i32,
    /// Child specific field.
    #[same_as(parent_table::parent_field)]
    #[same_as(satellite_table::field)]
    child_field: String,
    /// Field linking to mandatory table.
    #[mandatory(satellite_table)]
    mandatory_id: i32,
}

#[test]
fn test_mandatory_vertical_same_as() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS parent_table (
            id INTEGER PRIMARY KEY,
            parent_field TEXT NOT NULL,
            UNIQUE (id, parent_field)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS satellite_table (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER NOT NULL REFERENCES parent_table(id),
            field TEXT NOT NULL,
            UNIQUE (id, field),
            UNIQUE (id, parent_id)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS child_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            child_field TEXT NOT NULL,
            mandatory_id INTEGER NOT NULL REFERENCES satellite_table(id),
            FOREIGN KEY (id, child_field) REFERENCES parent_table(id, parent_field),
            FOREIGN KEY (mandatory_id, id) REFERENCES satellite_table(id, parent_id),
            FOREIGN KEY (mandatory_id, child_field) REFERENCES satellite_table(id, field)
        )",
    )
    .execute(&mut conn)?;

    let child = child_table::table::builder()
        .mandatory(satellite_table::table::builder().field("Mandatory Value"))
        .insert(&mut conn)?;

    let parent: Parent = child.ancestor(&mut conn)?;
    assert_eq!(parent.parent_field(), "Mandatory Value");

    // Test iter_foreign_keys with composite index (satellite_table::id,
    // satellite_table::parent_id)
    let mandatory = child.mandatory(&mut conn)?;
    let refs: Vec<_> =
        child.iter_foreign_keys::<(satellite_table::id, satellite_table::parent_id)>().collect();
    assert_eq!(refs.len(), 1);
    assert!(
        refs.contains(&(&mandatory.get_column::<satellite_table::id>(), mandatory.parent_id()))
    );

    Ok(())
}
