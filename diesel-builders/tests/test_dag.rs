//! Test case for foreign key based inheritance where the dependencies
//! form a directed acyclic graph (DAG).
//!
//! We have a root table A, which has two descendants B and C. Both B and C
//! extend A via foreign keys. Then, we have a table D that extends both B and C
//! via foreign keys. Each table as a simple column in addition to the primary
//! key to avoid having an excessively trivial test case.

mod common;

use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::{
    Decoupled, GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel,
};

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

// Define table B (extends A)
diesel::table! {
    /// Table B extends A via foreign key.
    table_b (id) {
        /// Primary key of table B, foreign key to table_a.id.
        id -> Integer,
        /// A simple column for table B.
        column_b -> Text,
    }
}

// Define table C (extends A)
diesel::table! {
    /// Table C extends A via foreign key.
    table_c (id) {
        /// Primary key of table C, foreign key to table_a.id.
        id -> Integer,
        /// A simple column for table C.
        column_c -> Text,
    }
}

// Define table D (extends both B and C)
diesel::table! {
    /// Table D extends both B and C via foreign keys.
    table_d (id) {
        /// Primary key of table D.
        id -> Integer,
        /// A simple column for table D.
        column_d -> Text,
    }
}

// Allow tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(table_a, table_b, table_c, table_d);

// Table A models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = table_a)]
/// A model for table A.
pub struct TableA {
    id: i32,
    column_a: String,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_a)]
/// A new model for insertions into table A.
pub struct NewTableA {
    column_a: Option<String>,
}

impl TableAddition for table_a::table {
    type InsertableModel = NewTableA;
    type Model = TableA;
    type InsertableColumns = (table_a::column_a,);
}

// Table B models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel, Decoupled,
)]
#[diesel(table_name = table_b)]
/// A model for table B.
pub struct TableB {
    id: i32,
    column_b: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_b)]
/// A new model for insertions into table B.
pub struct NewTableB {
    id: Option<i32>,
    column_b: Option<String>,
}

impl TableAddition for table_b::table {
    type InsertableModel = NewTableB;
    type Model = TableB;
    type InsertableColumns = (table_b::id, table_b::column_b);
}

// Table C models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel, Decoupled,
)]
#[diesel(table_name = table_c)]
/// A model for table C.
pub struct TableC {
    id: i32,
    column_c: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_c::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_c)]
/// A new model for insertions into table C.
pub struct NewTableC {
    id: Option<i32>,
    column_c: Option<String>,
}

impl TableAddition for table_c::table {
    type InsertableModel = NewTableC;
    type Model = TableC;
    type InsertableColumns = (table_c::id, table_c::column_c);
}

// Table D models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel, Decoupled,
)]
#[diesel(table_name = table_d)]
/// A model for table D.
pub struct TableD {
    id: i32,
    column_d: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_d::table {
    type Ancestors = (table_a::table, table_b::table, table_c::table);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_d)]
/// A new model for insertions into table D.
pub struct NewTableD {
    id: Option<i32>,
    column_d: Option<String>,
}

impl TableAddition for table_d::table {
    type InsertableModel = NewTableD;
    type Model = TableD;
    type InsertableColumns = (table_d::id, table_d::column_d);
}

#[test]
fn test_dag() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create table A
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Create table B (extends A)
    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_a(id),
            column_b TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Create table C (extends A)
    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_a(id),
            column_c TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Create table D (extends both B and C)
    diesel::sql_query(
        "CREATE TABLE table_d (
            id INTEGER PRIMARY KEY NOT NULL,
            column_d TEXT NOT NULL,
			FOREIGN KEY (id) REFERENCES table_b(id),
			FOREIGN KEY (id) REFERENCES table_c(id)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let a: TableA = table_a::table::builder()
        .set_column::<table_a::column_a>(&"Value A".to_string())
        .insert(&mut conn)
        .expect("Failed to insert into table A");

    assert_eq!(a.column_a, "Value A");

    // Insert into table B (extends A)
    let b: TableB = table_b::table::builder()
        .set_column::<table_a::column_a>(&"Value A for B".to_string())
        .set_column::<table_b::column_b>(&"Value B".to_string())
        .insert(&mut conn)
        .expect("Failed to insert into table B");

    assert_eq!(b.column_b, "Value B");

    // Insert into table C (extends A)
    let c: TableC = table_c::table::builder()
        .set_column::<table_a::column_a>(&"Value A for C".to_string())
        .set_column::<table_c::column_c>(&"Value C".to_string())
        .insert(&mut conn)
        .expect("Failed to insert into table C");

    assert_eq!(c.column_c, "Value C");

    // Insert into table D (extends both B and C)
    let mut d_builder = table_d::table::builder();
    d_builder
        .set_column_ref::<table_a::column_a>(&"Value A for D".to_string())
        .set_column_ref::<table_b::column_b>(&"Value B for D".to_string())
        .set_column_ref::<table_c::column_c>(&"Value C for D".to_string())
        .set_column_ref::<table_d::column_d>(&"Value D".to_string());

    // Test Debug formatting
    let _formatted = format!("{d_builder:?}");

    let d: TableD = d_builder
        .insert(&mut conn)
        .expect("Failed to insert into table D");

    assert_eq!(d.column_d, "Value D");

    // Query to verify relationships
    let queried_a: TableA = table_a::table
        .filter(table_a::id.eq(d.id))
        .first(&mut conn)?;
    assert_eq!(queried_a.column_a, "Value A for D");
    let queried_b: TableB = table_b::table
        .filter(table_b::id.eq(d.id))
        .first(&mut conn)?;
    assert_eq!(queried_b.column_b, "Value B for D");
    let queried_c: TableC = table_c::table
        .filter(table_c::id.eq(d.id))
        .first(&mut conn)?;
    assert_eq!(queried_c.column_c, "Value C for D");
    let queried_d: TableD = table_d::table
        .filter(table_d::id.eq(d.id))
        .first(&mut conn)?;
    assert_eq!(queried_d, d);

    Ok(())
}
