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

    // Idx = (nodes::id,)

    // Simple Iterator: Nested tuples of Options (Some(&T),)
    let simple_refs: Vec<_> = edge_instance.iter_match_simple::<(nodes::id,)>().collect();
    assert_eq!(simple_refs.len(), 2);
    // Nested tuple structure: (Val,)
    assert!(simple_refs.contains(&(Some(&1),)));
    assert!(simple_refs.contains(&(Some(&2),)));

    // Full Iterator: Nested tuples of Refs (&T,)
    let full_refs: Vec<_> = edge_instance.iter_match_full::<(nodes::id,)>().collect();
    assert_eq!(full_refs.len(), 2);
    assert!(full_refs.contains(&(&1,)));
    assert!(full_refs.contains(&(&2,)));
}

#[test]
fn test_iter_foreign_keys_with_optional_edges() {
    let edge_none = OptionalEdge { id: 1, source_id: None, target_id: None };

    // Simple: Yields (None,)
    let refs_none: Vec<_> = edge_none.iter_match_simple::<(nodes::id,)>().collect();
    assert_eq!(refs_none.len(), 2);
    assert!(refs_none.contains(&(None,)));

    // Full: Skips None
    let full_none: Vec<_> = edge_none.iter_match_full::<(nodes::id,)>().collect();
    assert_eq!(full_none.len(), 0);

    let edge_mixed = OptionalEdge { id: 2, source_id: Some(10), target_id: None };

    // Simple
    let refs_mixed: Vec<_> = edge_mixed.iter_match_simple::<(nodes::id,)>().collect();
    assert_eq!(refs_mixed.len(), 2);
    assert!(refs_mixed.contains(&(Some(&10),)));
    assert!(refs_mixed.contains(&(None,)));

    // Full
    let full_mixed: Vec<_> = edge_mixed.iter_match_full::<(nodes::id,)>().collect();
    assert_eq!(full_mixed.len(), 1);
    assert!(full_mixed.contains(&(&10,)));

    let edge_full = OptionalEdge { id: 3, source_id: Some(10), target_id: Some(20) };

    // Simple
    let refs_full: Vec<_> = edge_full.iter_match_simple::<(nodes::id,)>().collect();
    assert_eq!(refs_full.len(), 2);
    assert!(refs_full.contains(&(Some(&10),)));
    assert!(refs_full.contains(&(Some(&20),)));

    // Full
    let full_full: Vec<_> = edge_full.iter_match_full::<(nodes::id,)>().collect();
    assert_eq!(full_full.len(), 2);
    assert!(full_full.contains(&(&10,)));
    assert!(full_full.contains(&(&20,)));
}

#[test]
fn test_mixed_optional_edge() {
    let mixed = MixedOptionalEdge { id: 0, source_id: None, target_id: 2 };

    // Simple: (Option<&src>,), (Option<&tgt>,)
    let simple: Vec<_> = mixed.iter_match_simple::<(nodes::id,)>().collect();
    assert_eq!(simple.len(), 2);
    assert!(simple.contains(&(None,)));
    assert!(simple.contains(&(Some(&2),)));

    // Full: Only target_id is valid
    let full: Vec<_> = mixed.iter_match_full::<(nodes::id,)>().collect();
    assert_eq!(full.len(), 1);
    assert!(full.contains(&(&2,)));

    let mixed_both = MixedOptionalEdge { id: 1, source_id: Some(1), target_id: 2 };
    let full_both: Vec<_> = mixed_both.iter_match_full::<(nodes::id,)>().collect();
    assert_eq!(full_both.len(), 2);
    assert!(full_both.contains(&(&1,)));
    assert!(full_both.contains(&(&2,)));
}

#[test]
fn test_iter_foreign_key_columns() {
    // iter_foreign_key_columns matches both iterators logic in iteration order
    // But returns nested tuples of boxed columns
    let keys: Vec<_> = Edge::iter_foreign_key_columns::<(nodes::id,)>().collect();

    assert_eq!(keys.len(), 2);

    // keys[0] is (Box<Col>, NestedTail)
    // For single col FK, it is (Box<Col>, ())

    // Check column names.
    let col1_name = keys[0].0.column_name();
    let col2_name = keys[1].0.column_name();

    assert_eq!(col1_name, edges::source_id::NAME);
    assert_eq!(col2_name, edges::target_id::NAME);
}
