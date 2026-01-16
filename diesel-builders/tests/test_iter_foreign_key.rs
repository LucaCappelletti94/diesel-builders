//! Tests for iterating foreign keys.

use diesel::prelude::*;
use diesel_builders::{DynTypedColumn, IterForeignKeyExt, prelude::*};

/// Node table.
#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = nodes)]
#[table_model(surrogate_key)]
pub struct Node {
    /// ID
    id: i32,
    /// Name
    name: String,
}

/// Edge table with two FKs to Node.
#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = edges)]
#[table_model(surrogate_key)]
#[table_model(foreign_key(source_id, (nodes::id)))]
#[table_model(foreign_key(target_id, (nodes::id)))]
pub struct Edge {
    /// ID
    id: i32,
    /// Source Node ID
    source_id: i32,
    /// Target Node ID
    target_id: i32,
}

/// Optional Edge table with two optional FKs to Node.
#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = optional_edges)]
#[table_model(surrogate_key)]
#[table_model(foreign_key(source_id, (nodes::id)))]
#[table_model(foreign_key(target_id, (nodes::id)))]
pub struct OptionalEdge {
    /// ID
    id: i32,
    /// Optional Source Node ID
    source_id: Option<i32>,
    /// Optional Target Node ID
    target_id: Option<i32>,
}

/// Optional Edge table with two optional FKs to Node.
#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = mixed_optional_edges)]
#[table_model(surrogate_key)]
#[table_model(foreign_key(source_id, (nodes::id)))]
#[table_model(foreign_key(target_id, (nodes::id)))]
pub struct MixedOptionalEdge {
    /// ID
    id: i32,
    /// Optional Source Node ID
    source_id: Option<i32>,
    /// Optional Target Node ID
    target_id: i32,
}

#[test]
fn test_iter_foreign_keys_with_non_optional_edges() {
    let edge_instance = Edge { id: 0, source_id: 1, target_id: 2 };

    // Iterate foreign keys pointing to nodes::id
    // Idx = (nodes::id,) (the referenced index)
    let refs: Vec<_> = edge_instance.iter_foreign_keys::<(nodes::id,)>().collect();

    assert_eq!(refs.len(), 2);
    // FlattenNestedTuple for (i32,) is (i32,).
    assert!(refs.contains(&(&1,)));
    assert!(refs.contains(&(&2,)));
}

#[test]
fn test_iter_foreign_keys_with_optional_edges() {
    let edge_none = OptionalEdge { id: 1, source_id: None, target_id: None };
    let refs_none: Vec<_> = edge_none.iter_foreign_keys::<(nodes::id,)>().collect();
    // With no unwrapping/filtering, we get Option references
    assert_eq!(refs_none.len(), 2); // Both FKs exist, even if None
    assert!(refs_none.contains(&(None,)));

    let edge_mixed = OptionalEdge { id: 2, source_id: Some(10), target_id: None };
    let refs_mixed: Vec<_> = edge_mixed.iter_foreign_keys::<(nodes::id,)>().collect();
    assert_eq!(refs_mixed.len(), 2);
    assert!(refs_mixed.contains(&(Some(&10),)));
    assert!(refs_mixed.contains(&(None,)));

    let edge_full = OptionalEdge { id: 3, source_id: Some(10), target_id: Some(20) };
    let refs_full: Vec<_> = edge_full.iter_foreign_keys::<(nodes::id,)>().collect();
    assert_eq!(refs_full.len(), 2);
    assert!(refs_full.contains(&(Some(&10),)));
    assert!(refs_full.contains(&(Some(&20),)));
}

#[test]
fn test_iter_foreign_keys_edges() {
    let edge_instance = Edge { id: 0, source_id: 1, target_id: 2 };

    // Simple iterator includes all foreign keys (yields raw field references)
    let refs: Vec<_> = edge_instance.iter_foreign_keys::<(nodes::id,)>().collect();

    assert_eq!(refs.len(), 2);
    assert!(refs.contains(&(&1,)));
    assert!(refs.contains(&(&2,)));
}

#[test]
fn test_iter_foreign_keys_optional_edges() {
    // Simple iterator yields references to the Option fields

    let edge_full = OptionalEdge { id: 3, source_id: Some(10), target_id: Some(20) };
    let refs_full: Vec<_> = edge_full.iter_foreign_keys::<(nodes::id,)>().collect();
    assert_eq!(refs_full.len(), 2);
    assert!(refs_full.contains(&(Some(&10),)));
    assert!(refs_full.contains(&(Some(&20),)));
}

#[test]
fn test_iter_foreign_keys() {
    let edge_instance = Edge { id: 0, source_id: 1, target_id: 2 };

    // iter_foreign_key_columns returns boxed host table columns
    let keys: Vec<_> = edge_instance.iter_foreign_key_columns::<(nodes::id,)>().collect();

    // Should have 2 foreign keys (source_id and target_id both reference nodes::id)
    assert_eq!(keys.len(), 2);

    // The iterator returns tuples of boxed DynTypedColumn trait objects
    assert_ne!(keys[0].0.column_name(), nodes::id.column_name());
    assert_eq!(keys[0].0.column_name(), edges::source_id.column_name());
    assert_eq!(keys[1].0.column_name(), edges::target_id.column_name());
}

#[test]
fn test_iter_foreign_key_columns_with_optional_edges() {
    let edge_instance = OptionalEdge { id: 0, source_id: Some(1), target_id: Some(2) };

    // iter_foreign_key_columns returns the same column tuples regardless of values
    let keys: Vec<_> = edge_instance.iter_foreign_key_columns::<(nodes::id,)>().collect();

    // Should have 2 foreign keys
    assert_eq!(keys.len(), 2);

    // Verify the columns are from the host table (optional_edges), not the
    // referenced table (nodes)
    assert_eq!(keys[0].0.column_name(), optional_edges::source_id.column_name());
    assert_eq!(keys[1].0.column_name(), optional_edges::target_id.column_name());

    // Test with None values - should return the same columns
    let edge_none = OptionalEdge { id: 1, source_id: None, target_id: None };
    let keys_none: Vec<_> = edge_none.iter_foreign_key_columns::<(nodes::id,)>().collect();
    assert_eq!(keys_none.len(), 2);
    assert_eq!(keys_none[0].0.column_name(), optional_edges::source_id.column_name());
    assert_eq!(keys_none[1].0.column_name(), optional_edges::target_id.column_name());
}
