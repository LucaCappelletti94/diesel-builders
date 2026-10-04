//! Explicit relation keys propagate primary keys to distinct mandatory tables.

use diesel_builders::prelude::*;

#[derive(Clone, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = parents)]
#[table_model(surrogate_key)]
/// A parent record.
pub struct Parent {
    /// Parent identity.
    id: i32,
    /// Parent label.
    name: String,
}

#[derive(Clone, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = first_sides)]
#[table_model(surrogate_key, foreign_key(parent_id, (parents::id)))]
/// The first mandatory record.
pub struct FirstSide {
    /// Side identity.
    id: i32,
    /// Owning parent.
    parent_id: i32,
    /// Side label.
    name: String,
}

#[derive(Clone, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = second_sides)]
#[table_model(surrogate_key, foreign_key(parent_id, (parents::id)))]
/// The second mandatory record.
pub struct SecondSide {
    /// Side identity.
    id: i32,
    /// Owning parent.
    parent_id: i32,
    /// Side label.
    name: String,
}

unique_index!(first_sides::id, first_sides::parent_id);
unique_index!(second_sides::id, second_sides::parent_id);

#[derive(Clone, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = children)]
#[table_model(ancestors(parents))]
/// A child with two keyed mandatory records.
pub struct Child {
    /// Parent identity.
    #[same_as(first_sides::parent_id, a_id)]
    #[same_as(second_sides::parent_id, b_id)]
    id: i32,
    /// First mandatory record.
    #[mandatory(first_sides)]
    a_id: i32,
    /// Second mandatory record.
    #[mandatory(second_sides)]
    b_id: i32,
    /// Child label.
    note: String,
}

diesel::allow_tables_to_appear_in_same_query!(parents, first_sides, second_sides, children);

#[test]
fn test_distinct_explicit_mandatory_keys() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;
    for statement in [
        "PRAGMA foreign_keys=ON",
        "CREATE TABLE parents(id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
        "CREATE TABLE first_sides(id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parents(id), name TEXT NOT NULL, UNIQUE(id, parent_id))",
        "CREATE TABLE second_sides(id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parents(id), name TEXT NOT NULL, UNIQUE(id, parent_id))",
        "CREATE TABLE children(id INTEGER PRIMARY KEY REFERENCES parents(id), a_id INTEGER NOT NULL, b_id INTEGER NOT NULL, note TEXT NOT NULL, FOREIGN KEY(a_id, id) REFERENCES first_sides(id, parent_id), FOREIGN KEY(b_id, id) REFERENCES second_sides(id, parent_id))",
    ] {
        diesel::sql_query(statement).execute(&mut conn)?;
    }
    let seed = parents::table::builder().name("Seed").insert(&mut conn)?;
    first_sides::table::builder().parent_id(*seed.id()).name("Existing first").insert(&mut conn)?;
    let child = children::table::builder()
        .name("Parent")
        .note("Child")
        .a(first_sides::table::builder().name("First"))
        .b(second_sides::table::builder().name("Second"))
        .insert(&mut conn)?;
    let parent: Parent = child.ancestor(&mut conn)?;
    let first = child.a(&mut conn)?;
    let second = child.b(&mut conn)?;
    assert_ne!(child.a_id(), child.b_id());
    assert_eq!(parent.name(), "Parent");
    assert_eq!(first.name(), "First");
    assert_eq!(second.name(), "Second");
    assert_eq!(first.parent_id(), parent.id());
    assert_eq!(second.parent_id(), parent.id());
    Ok(())
}
