//! Tests for iterating foreign keys.

use diesel::prelude::*;
use diesel_builders::{IterForeignKeyExt, prelude::*};
mod shared;

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

/// `EdgeType` table.
#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = edge_types)]
#[table_model(surrogate_key)]
pub struct EdgeType {
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
#[diesel(table_name = heterogenous_edges)]
#[table_model(ancestors(edges))]
#[table_model(foreign_key(edge_type_id, (edge_types::id)))]
#[table_model(foreign_key(another_node_id, (nodes::id)))]
pub struct HeterogenousEdge {
    /// ID
    id: i32,
    /// Type of Edge
    edge_type_id: i32,
    /// Another node ID
    another_node_id: Option<i32>,
}

#[test]
fn test_iter_foreign_keys_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    type NestedType = (Edge, (HeterogenousEdge,));

    let mut conn = shared::establish_connection()?;

    diesel::sql_query("CREATE TABLE nodes (id INTEGER PRIMARY KEY, name TEXT NOT NULL);")
        .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE edges (
            id INTEGER PRIMARY KEY,
            source_id INTEGER NOT NULL REFERENCES nodes(id),
            target_id INTEGER NOT NULL REFERENCES nodes(id)
        );",
    )
    .execute(&mut conn)?;
    diesel::sql_query("CREATE TABLE edge_types (id INTEGER PRIMARY KEY, name TEXT NOT NULL);")
        .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE heterogenous_edges (
            id INTEGER PRIMARY KEY REFERENCES edges(id),
            edge_type_id INTEGER NOT NULL REFERENCES edge_types(id),
            another_node_id INTEGER REFERENCES nodes(id)
        );",
    )
    .execute(&mut conn)?;

    let fk_columns: Vec<_> =
        <HeterogenousEdge as IterForeignKeyExt>::iter_foreign_key_columns::<(edge_types::id,)>()
            .collect();

    assert_eq!(fk_columns, vec![(heterogenous_edges::edge_type_id.into(),),]);

    let fk_matches: Vec<_> = <NestedType as IterForeignKeyExt>::iter_dynamic_foreign_key_columns((
        edge_types::id.into(),
    ))
    .collect();

    assert_eq!(fk_matches, vec![(heterogenous_edges::edge_type_id.into(),),]);

    // Since both `Edge` and `HeterogenousEdge` have foreign keys to `nodes::id`,
    // we should get two matches when iterating over foreign key matches for
    // `nodes::id`.
    let fk_node_matches: Vec<_> =
        <NestedType as IterForeignKeyExt>::iter_foreign_key_columns::<(nodes::id,)>().collect();
    assert_eq!(
        fk_node_matches,
        vec![
            (edges::source_id.into(),),
            (edges::target_id.into(),),
            (heterogenous_edges::another_node_id.into(),),
        ]
    );

    Ok(())
}
