//! Test case for checking that vertical same-as dependencies
//! work correctly in the context where the exist a mandatory
//! table setting the same column value.

mod shared;

use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = parent_table)]
#[table_model(surrogate_key)]
/// A parent table model.
pub struct Parent {
    /// Primary key.
    id: i32,
    /// A field in the parent table.
    parent_field: String,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = mandatory_table)]
#[table_model(surrogate_key)]
/// A parent table model.
pub struct Mandatory {
    /// Primary key.
    id: i32,
    /// The parent table id.
    parent_id: i32,
    /// A field in the parent table.
    mandatory_field: String,
}

unique_index!(mandatory_table::id, mandatory_table::mandatory_field);
unique_index!(mandatory_table::id, mandatory_table::parent_id);

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
/// Model for a child table that inherits from `parent_table`.
pub struct Child {
    #[same_as(mandatory_table::parent_id)]
    /// Primary key.
    id: i32,
    /// Child specific field.
    #[same_as(parent_table::parent_field)]
    #[same_as(mandatory_table::mandatory_field)]
    child_field: String,
    /// Field linking to mandatory table.
    #[mandatory(mandatory_table)]
    mandatory_id: i32,
}

fk!((child_table::mandatory_id, child_table::id) -> (mandatory_table::id, mandatory_table::parent_id));
fk!((child_table::mandatory_id, child_table::child_field) -> (mandatory_table::id, mandatory_table::mandatory_field));

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
        "CREATE TABLE IF NOT EXISTS mandatory_table (
            id INTEGER PRIMARY KEY,
            parent_id INTEGER NOT NULL REFERENCES parent_table(id),
            mandatory_field TEXT NOT NULL,
            UNIQUE (id, mandatory_field),
            UNIQUE (id, parent_id)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS child_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            child_field TEXT NOT NULL,
            mandatory_id INTEGER NOT NULL REFERENCES mandatory_table(id),
            FOREIGN KEY (id, child_field) REFERENCES parent_table(id, parent_field),
            FOREIGN KEY (mandatory_id, id) REFERENCES mandatory_table(id, parent_id),
            FOREIGN KEY (mandatory_id, child_field) REFERENCES mandatory_table(id, mandatory_field)
        )",
    )
    .execute(&mut conn)?;

    let child = child_table::table::builder()
        .mandatory(mandatory_table::table::builder().mandatory_field("Mandatory Value"))
        .insert(&mut conn)?;

    let parent: Parent = child.ancestor(&mut conn)?;
    assert_eq!(parent.parent_field(), "Mandatory Value");

    Ok(())
}
