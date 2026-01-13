//! Tests for iterating foreign keys.

use diesel::prelude::*;
use diesel_builders::{IterForeignKeyExt, prelude::*};

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

#[test]
fn test_iter_foreign_key_edges() {
    let edge_instance = Edge { id: 0, source_id: 1, target_id: 2 };

    // Iterate foreign keys pointing to nodes::id
    // Idx type is (nodes::id,)
    let refs: Vec<_> = edge_instance.iter_foreign_key::<(nodes::id,)>().collect();

    assert_eq!(refs.len(), 2);
    // FlattenNestedTuple for (i32,) is (i32,).
    assert!(refs.contains(&(&1,)));
    assert!(refs.contains(&(&2,)));
}

#[test]
fn test_iter_foreign_key_optional_edges() {
    let edge_none = OptionalEdge { id: 1, source_id: None, target_id: None };
    let refs_none: Vec<_> = edge_none.iter_foreign_key::<(nodes::id,)>().collect();
    assert!(refs_none.is_empty());

    let edge_mixed = OptionalEdge { id: 2, source_id: Some(10), target_id: None };
    let refs_mixed: Vec<_> = edge_mixed.iter_foreign_key::<(nodes::id,)>().collect();
    assert_eq!(refs_mixed.len(), 1);
    assert!(refs_mixed.contains(&(&10,)));

    let edge_full = OptionalEdge { id: 3, source_id: Some(10), target_id: Some(20) };
    let refs_full: Vec<_> = edge_full.iter_foreign_key::<(nodes::id,)>().collect();
    assert_eq!(refs_full.len(), 2);
    assert!(refs_full.contains(&(&10,)));
    assert!(refs_full.contains(&(&20,)));
}
