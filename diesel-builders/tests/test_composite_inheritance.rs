//! Composite ancestry uses every key component for insertion and loading.

use diesel_builders::{load_nested_query_builder::LoadNestedFirst, prelude::*};

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = composite_parents, primary_key(a_id, b_id))]
/// A composite-key parent.
pub struct Parent {
    /// First identity component.
    a_id: i32,
    /// Second identity component.
    b_id: i32,
    /// Parent label.
    name: String,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = composite_children, primary_key(a_id, b_id))]
#[table_model(ancestors(composite_parents))]
/// A composite-key descendant.
pub struct Child {
    /// First identity component.
    a_id: i32,
    /// Second identity component.
    b_id: i32,
    /// Child label.
    note: String,
}

diesel::allow_tables_to_appear_in_same_query!(composite_parents, composite_children);

#[test]
fn test_complete_composite_ancestry() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;
    for statement in [
        "PRAGMA foreign_keys=ON",
        "CREATE TABLE composite_parents(a_id INTEGER NOT NULL, b_id INTEGER NOT NULL, name TEXT NOT NULL, PRIMARY KEY(a_id, b_id))",
        "CREATE TABLE composite_children(a_id INTEGER NOT NULL, b_id INTEGER NOT NULL, note TEXT NOT NULL, PRIMARY KEY(a_id, b_id), FOREIGN KEY(a_id, b_id) REFERENCES composite_parents(a_id, b_id))",
    ] {
        diesel::sql_query(statement).execute(&mut conn)?;
    }
    for (a, b, name, note) in [
        (7, 11, "First parent", "First child"),
        (7, 12, "Second parent", "Second child"),
        (8, 11, "Third parent", "Third child"),
    ] {
        let builder =
            SetParentBId::b_id(SetParentAId::a_id(composite_children::table::builder(), a), b);
        let child = builder.name(name).note(note).insert(&mut conn)?;
        let parent: Parent = child.ancestor(&mut conn)?;
        let (loaded_parent, (descendant,)) =
            <(composite_parents::a_id, (composite_parents::b_id,)) as LoadNestedFirst<
                composite_children::table,
                _,
            >>::load_nested_first((*parent.a_id(), (*parent.b_id(),)), &mut conn)?;
        assert_eq!(loaded_parent, parent);
        assert_eq!((parent.a_id(), parent.b_id()), (&a, &b));
        assert_eq!(parent.name(), name);
        assert_eq!(descendant, child);
        assert_eq!(Child::find(child.id(), &mut conn)?, child);
        assert!(
            child
                .iter_match_full::<(composite_parents::a_id, composite_parents::b_id,)>()
                .eq(std::iter::once((&a, (&b,))))
        );
    }
    let builder =
        SetParentBId::b_id(SetParentAId::a_id(composite_children::table::builder(), 9), 14);
    let (parent, (child,)) =
        builder.name("Nested parent").note("Nested child").insert_nested(&mut conn)?;
    assert_eq!((parent.a_id(), parent.b_id()), (&9, &14));
    assert_eq!((child.a_id(), child.b_id()), (&9, &14));
    assert_eq!(parent.name(), "Nested parent");
    assert_eq!(child.note(), "Nested child");
    Ok(())
}
