//! Submodule to test a discretionary triangular relation between tables.
//!
//! This test sets up three tables: A, B, and C. B extends C, and C contains
//! a column that references A, and B has a column that references C, forming a
//! triangular relationship. The test verifies that inserts and queries work
//! correctly through this relationship.
//!
//! Specifically, the relationship is discretionary, that is the foreign key
//! from C to A is NOT referenced in B using a same-as relationship, which means
//! that the C record associated with a B record may reference the same A record
//! as B does, but it is not required to and the user can choose to set it or
//! not. Additionally, there exist a same-as relationship between B and C on
//! another column, which means that when setting the builder for the C record
//! in the B builder, that column value needs to be set, and when it is not set
//! by setting the associated C builder, it must be set manually in B.

mod common;

use diesel::prelude::*;
use diesel_additions::{SetColumnExt, TableAddition};
use diesel_builders::{
    BuildableTable, BundlableTable, NestedInsert, SetDiscretionaryBuilderExt,
    SetDiscretionaryModelExt,
};
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel};
use diesel_relations::Descendant;

// Define table A (root table)
diesel::table! {
    /// Root table A.
    table_a (id) {
        /// Primary key of table A.
        id -> Integer,
        /// A simple column for table A.
        column_a -> Text,
    }
}

// Define table C (has a foreign key to A)
diesel::table! {
    /// Table C with a foreign key to A.
    table_c (id) {
        /// Primary key of table C.
        id -> Integer,
        /// Foreign key to table A.
        a_id -> Integer,
        /// A simple column for table C.
        column_c -> Text,
    }
}

// Define table B (extends C and has its own foreign key to A)
diesel::table! {
    /// Table B extends C via foreign key and also references A.
    table_b (id) {
        /// Primary key of table B, foreign key to table_c.id.
        id -> Integer,
        /// Foreign key to table C (must match C's a_id - discretionary triangular relation).
        c_id -> Integer,
        /// A simple column for table B.
        column_b -> Text,
        /// The remote column_c value from table C that B references via c_id.
        remote_column_c -> Text,
    }
}

// Define join relationships
diesel::joinable!(table_c -> table_a (a_id));
diesel::joinable!(table_b -> table_c (id));
diesel::joinable!(table_b -> table_a (c_id));

// Allow tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(table_a, table_b, table_c);

// Table A models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = table_a)]
/// Model for table A.
pub struct TableA {
    /// Primary key.
    pub id: i32,
    /// Column A value.
    pub column_a: String,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_a)]
/// Insertable model for table A.
pub struct NewTableA {
    /// Column A value.
    pub column_a: Option<String>,
}

impl TableAddition for table_a::table {
    type InsertableModel = NewTableA;
    type Model = TableA;
    type InsertableColumns = (table_a::column_a,);
}

// Table C models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = table_c)]
/// Model for table C.
pub struct TableC {
    /// Primary key.
    pub id: i32,
    /// Foreign key to table A.
    pub a_id: i32,
    /// Column C value.
    pub column_c: String,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_c)]
/// Insertable model for table C.
pub struct NewTableC {
    /// Foreign key to table A.
    pub a_id: Option<i32>,
    /// Column C value.
    pub column_c: Option<String>,
}

impl TableAddition for table_c::table {
    type InsertableModel = NewTableC;
    type Model = TableC;
    type InsertableColumns = (table_c::a_id, table_c::column_c);
}

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel)]
#[diesel(table_name = table_b)]
/// Model for table B.
pub struct TableB {
    /// Primary key.
    pub id: i32,
    /// Foreign key to table A.
    pub c_id: i32,
    /// Column B value.
    pub column_b: String,
    /// The remote column_c value from table C that B references via c_id.
    pub remote_column_c: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_b)]
/// Insertable model for table B.
pub struct NewTableB {
    /// Primary key.
    pub id: Option<i32>,
    /// Foreign key to table C.
    pub c_id: Option<i32>,
    /// Column B value.
    pub column_b: Option<String>,
    /// The remote column_c value from table C that B references via c_id.
    pub remote_column_c: Option<String>,
}

impl TableAddition for table_b::table {
    type InsertableModel = NewTableB;
    type Model = TableB;
    type InsertableColumns =
        (table_b::id, table_b::c_id, table_b::column_b, table_b::remote_column_c);
}

// Implement SingletonForeignKey for table_b::c_id to indicate it references
// table_c
impl diesel_additions::SingletonForeignKey for table_b::c_id {
    type ReferencedTable = table_c::table;
}

// This is the key part: B's c_id must match C's id, and C's a_id must match A's
// id. We express that B's c_id is horizontally the same as C's a_id, which in
// turn is the same as A's id.
impl diesel_relations::HorizontalSameAsKey for table_b::c_id {
    // HostColumns are columns in table_b (the same table) that relate to this key
    // In this case, there are no other columns in table_b that need to match
    // Actually, we need to think about this differently...
    type HostColumns = (table_b::id, table_b::remote_column_c);
    type ForeignColumns = (table_c::a_id, table_c::column_c);
}

#[diesel_builders_macros::bundlable_table]
impl BundlableTable for table_b::table {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = (table_b::c_id,);
}

#[test]
fn test_discretionary_triangular_relation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create table A
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Create table C (references A)
    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL REFERENCES table_a(id),
            column_c TEXT NOT NULL,
			UNIQUE(id, column_c)
        )",
    )
    .execute(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_a(id),
            c_id INTEGER NOT NULL REFERENCES table_c(id),
            column_b TEXT NOT NULL,
            remote_column_c TEXT NOT NULL,
			FOREIGN KEY (c_id, remote_column_c) REFERENCES table_c(id, column_c)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let a = table_a::table::builder()
        .set_column::<table_a::column_a>(&"Value A".to_string())
        .insert(&mut conn)?;

    assert_eq!(a.column_a, "Value A");

    // Insert into table C (references A)
    let c = table_c::table::builder()
        .set_column::<table_c::a_id>(&a.id)
        .set_column::<table_c::column_c>(&"Value C".to_string())
        .insert(&mut conn)?;

    assert_eq!(c.column_c, "Value C");
    assert_eq!(c.a_id, a.id);

    let mut c_builder = table_c::table::builder();
    c_builder
        .set_column::<table_c::a_id>(&a.id)
        .set_column::<table_c::column_c>(&"Value C for B".to_string());

    // Insert into table B (extends C and references A)
    // The discretionary triangular relation means B's a_id should automatically
    // match C's a_id when we only set C's columns
    let triangular_b = table_b::table::builder()
        .set_column::<table_a::column_a>(&"Value A for B".to_string())
        .set_column::<table_b::column_b>(&"Value B".to_string())
        .set_discretionary_builder::<table_b::c_id>(c_builder)
        .insert(&mut conn)?;

    let associated_a: TableA =
        table_a::table.filter(table_a::id.eq(triangular_b.id)).first(&mut conn).unwrap();
    assert_eq!(associated_a.column_a, "Value A for B");

    let associated_c: TableC =
        table_c::table.filter(table_c::id.eq(triangular_b.c_id)).first(&mut conn).unwrap();
    assert_eq!(associated_c.column_c, "Value C for B");
    assert_eq!(associated_c.a_id, triangular_b.id);
    assert_eq!(associated_c.a_id, associated_a.id);

    let indipendent_b = table_b::table::builder()
        .set_column::<table_a::column_a>(&"Independent A for B".to_string())
        .set_column::<table_b::column_b>(&"Independent B".to_string())
        .set_discretionary_model::<table_b::c_id>(&c)
        .insert(&mut conn)?;

    assert_eq!(indipendent_b.column_b, "Independent B");
    assert_eq!(indipendent_b.remote_column_c, "Value C");
    assert_ne!(indipendent_b.id, triangular_b.id);
    assert_ne!(indipendent_b.id, c.a_id);

    Ok(())
}
