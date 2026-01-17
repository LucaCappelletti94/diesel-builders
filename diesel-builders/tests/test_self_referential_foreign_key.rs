//! Tests for self-referential foreign keys (taxonomy-like structures).

use diesel::prelude::*;
use diesel_builders::prelude::*;

/// A taxonomy table with an optional `parent_id` that references itself
#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = taxonomy)]
#[table_model(surrogate_key)]
#[table_model(foreign_key(parent_id, (taxonomy::id)))]
pub struct Taxonomy {
    /// Primary key.
    id: i32,
    /// The name of the taxonomy category.
    name: String,
    /// Optional parent taxonomy ID (self-referential foreign key).
    parent_id: Option<i32>,
}

mod shared {
    use super::*;

    pub fn establish_connection() -> SqliteConnection {
        let mut conn = SqliteConnection::establish(":memory:")
            .expect("Failed to establish in-memory SQLite connection");

        // Create taxonomy table with self-referential foreign key
        diesel::sql_query(
            "CREATE TABLE taxonomy (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES taxonomy(id)
            );",
        )
        .execute(&mut conn)
        .expect("Failed to create taxonomy table");

        conn
    }
}

#[test]
fn test_taxonomy_root_node() {
    let mut conn = shared::establish_connection();

    // Create a root taxonomy node (no parent)
    let builder = taxonomy::table::builder().name("Root Category");

    let builder_clone = builder.clone();
    let root = builder.insert(&mut conn).expect("Failed to insert root taxonomy");

    let nested_models =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested root taxonomy");
    assert_eq!(nested_models.name(), root.name());
    assert_eq!(nested_models.parent_id(), root.parent_id());

    assert_eq!(*root.name(), "Root Category");
    assert_eq!(root.parent_id(), &None);

    // Verify we can query it back
    let loaded: Taxonomy = taxonomy::table
        .find(root.get_column_ref::<taxonomy::id>())
        .first(&mut conn)
        .expect("Failed to load root taxonomy");

    assert_eq!(loaded, root);
}

#[test]
fn test_taxonomy_with_parent() {
    let mut conn = shared::establish_connection();

    // Create a root node
    let builder = taxonomy::table::builder().name("Electronics");
    let builder_clone = builder.clone();
    let root = builder.insert(&mut conn).expect("Failed to insert root");

    let nested_models =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested root");
    assert_eq!(nested_models.name(), root.name());

    // Create a child node referencing the root
    let builder = taxonomy::table::builder()
        .name("Computers")
        .parent_id(Some(root.get_column::<taxonomy::id>()));

    let builder_clone = builder.clone();
    let child = builder.insert(&mut conn).expect("Failed to insert child");

    let nested_models =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested child");
    assert_eq!(nested_models.name(), child.name());
    assert_eq!(nested_models.parent_id(), child.parent_id());

    assert_eq!(*child.name(), "Computers");
    assert_eq!(child.parent_id(), &Some(root.get_column::<taxonomy::id>()));

    // Verify parent relationship
    assert_eq!(child.parent_id(), &Some(root.get_column::<taxonomy::id>()));

    // Test iter_foreign_keys - self-referential FK
    let refs: Vec<_> = child.iter_match_simple::<(taxonomy::id,)>().collect();
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(&(Some(root.get_column_ref::<taxonomy::id>()),)));
}

#[test]
fn test_taxonomy_hierarchy() {
    let mut conn = shared::establish_connection();

    // Create a three-level hierarchy: Root -> Category -> Subcategory
    let builder = taxonomy::table::builder().name("Products");
    let builder_clone = builder.clone();
    let root = builder.insert(&mut conn).expect("Failed to insert root");

    let nested_models =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested root");
    assert_eq!(nested_models.name(), root.name());

    let builder = taxonomy::table::builder()
        .name("Electronics")
        .parent_id(Some(root.get_column::<taxonomy::id>()));
    let builder_clone = builder.clone();
    let category = builder.insert(&mut conn).expect("Failed to insert category");

    let nested_models =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested category");
    assert_eq!(nested_models.name(), category.name());

    let builder = taxonomy::table::builder()
        .name("Laptops")
        .parent_id(Some(category.get_column::<taxonomy::id>()));
    let builder_clone = builder.clone();
    let subcategory = builder.insert(&mut conn).expect("Failed to insert subcategory");

    let nested_models =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested subcategory");
    assert_eq!(nested_models.name(), subcategory.name());

    // Verify the hierarchy
    assert_eq!(root.parent_id(), &None);
    assert_eq!(category.parent_id(), &Some(root.get_column::<taxonomy::id>()));
    assert_eq!(subcategory.parent_id(), &Some(category.get_column::<taxonomy::id>()));

    // Query all nodes
    let all_nodes: Vec<Taxonomy> =
        taxonomy::table.load(&mut conn).expect("Failed to load all taxonomy nodes");

    // We inserted 3 nodes normally, and 3 via insert_nested (duplicates)
    assert_eq!(all_nodes.len(), 6);
}

#[test]
fn test_taxonomy_multiple_children() {
    let mut conn = shared::establish_connection();

    // Create a root with multiple children
    let builder = taxonomy::table::builder().name("Animals");
    let builder_clone = builder.clone();
    let root = builder.insert(&mut conn).expect("Failed to insert root");

    let nested_root = builder_clone.insert_nested(&mut conn).expect("Failed to insert nested root");
    assert_eq!(nested_root.name(), root.name());

    let builder = taxonomy::table::builder()
        .name("Mammals")
        .parent_id(Some(root.get_column::<taxonomy::id>()));
    let builder_clone = builder.clone();
    let child1 = builder.insert(&mut conn).expect("Failed to insert child1");

    let nested_child1 =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested child1");
    assert_eq!(nested_child1.name(), child1.name());

    let builder =
        taxonomy::table::builder().name("Birds").parent_id(Some(root.get_column::<taxonomy::id>()));
    let builder_clone = builder.clone();
    let child2 = builder.insert(&mut conn).expect("Failed to insert child2");

    let nested_child2 =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested child2");
    assert_eq!(nested_child2.name(), child2.name());

    let builder = taxonomy::table::builder()
        .name("Reptiles")
        .parent_id(Some(root.get_column::<taxonomy::id>()));
    let builder_clone = builder.clone();
    let child3 = builder.insert(&mut conn).expect("Failed to insert child3");

    let nested_child3 =
        builder_clone.insert_nested(&mut conn).expect("Failed to insert nested child3");
    assert_eq!(nested_child3.name(), child3.name());

    // Verify all children have the same parent
    assert_eq!(child1.parent_id(), &Some(root.get_column::<taxonomy::id>()));
    assert_eq!(child2.parent_id(), &Some(root.get_column::<taxonomy::id>()));
    assert_eq!(child3.parent_id(), &Some(root.get_column::<taxonomy::id>()));

    // Query children using the parent_id
    let children: Vec<Taxonomy> = taxonomy::table
        .filter(taxonomy::parent_id.eq(root.get_column::<taxonomy::id>()))
        .load(&mut conn)
        .expect("Failed to load children");

    assert_eq!(children.len(), 6);
}

#[test]
fn test_taxonomy_update_parent() {
    let mut conn = shared::establish_connection();

    // Create two root nodes
    let root1 = taxonomy::table::builder()
        .name("Category A")
        .insert(&mut conn)
        .expect("Failed to insert root1");

    let root2 = taxonomy::table::builder()
        .name("Category B")
        .insert(&mut conn)
        .expect("Failed to insert root2");

    // Create a child under root1
    let child = taxonomy::table::builder()
        .name("Subcategory")
        .parent_id(Some(root1.get_column::<taxonomy::id>()))
        .insert(&mut conn)
        .expect("Failed to insert child");

    assert_eq!(child.parent_id(), &Some(root1.get_column::<taxonomy::id>()));

    // Move the child to root2
    diesel::update(taxonomy::table.find(child.get_column::<taxonomy::id>()))
        .set(taxonomy::parent_id.eq(root2.get_column::<taxonomy::id>()))
        .execute(&mut conn)
        .expect("Failed to update parent");

    let updated_child: Taxonomy = taxonomy::table
        .find(child.get_column::<taxonomy::id>())
        .first(&mut conn)
        .expect("Failed to load updated child");

    assert_eq!(updated_child.parent_id(), &Some(root2.get_column::<taxonomy::id>()));
}

#[test]
fn test_taxonomy_orphan_node() {
    let mut conn = shared::establish_connection();

    // Create a node with a parent
    let parent = taxonomy::table::builder()
        .name("Parent")
        .insert(&mut conn)
        .expect("Failed to insert parent");

    let child = taxonomy::table::builder()
        .name("Child")
        .parent_id(Some(parent.get_column::<taxonomy::id>()))
        .insert(&mut conn)
        .expect("Failed to insert child");

    // Make the child an orphan by setting parent_id to None
    diesel::update(taxonomy::table.find(child.get_column::<taxonomy::id>()))
        .set(taxonomy::parent_id.eq(None::<i32>))
        .execute(&mut conn)
        .expect("Failed to orphan child");

    let orphan: Taxonomy = taxonomy::table
        .find(child.get_column::<taxonomy::id>())
        .first(&mut conn)
        .expect("Failed to load orphaned child");

    assert_eq!(orphan.parent_id(), &None);
}
