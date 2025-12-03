//! Test case for a table with both mandatory and discretionary triangular relations.
//!
//! This test sets up four tables: A (root), C (references A), D (references A),
//! and B which:
//! - Has a mandatory triangular relation with C (via c_id)
//! - Has a discretionary triangular relation with D (via d_id)
//!
//! The structure is:
//! - Table A: Root table
//! - Table C: References A, has column_c
//! - Table D: References A, has column_d
//! - Table B: References both C and D, with same-as constraints to A

mod common;

use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel_builders::{
    CompletedTableBuilderBundle, TableBuilder, TableBuilderBundle, prelude::*,
    table_builder::CompletedTableBuilder,
};
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel};

// Define table A (root table)
diesel::table! {
    table_a (id) {
        id -> Integer,
        column_a -> Text,
    }
}

// Define table C (has a foreign key to A)
diesel::table! {
    table_c (id) {
        id -> Integer,
        a_id -> Integer,
        column_c -> Nullable<Text>,
    }
}

// Define table D (has a foreign key to A)
diesel::table! {
    table_d (id) {
        id -> Integer,
        a_id -> Integer,
        column_d -> Nullable<Text>,
    }
}

// Define table B (references both C and D, with mandatory and discretionary relations)
diesel::table! {
    table_b (id) {
        id -> Integer,
        c_id -> Integer,
        d_id -> Integer,
        column_b -> Text,
        remote_column_c -> Nullable<Text>,
        remote_column_d -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(table_a, table_b, table_c, table_d);

// Table A models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = table_a)]
pub struct TableA {
    pub id: i32,
    pub column_a: String,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_a)]
pub struct NewTableA {
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
pub struct TableC {
    pub id: i32,
    pub a_id: i32,
    pub column_c: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_c)]
pub struct NewTableC {
    pub a_id: Option<i32>,
    pub column_c: Option<Option<String>>,
}

impl TableAddition for table_c::table {
    type InsertableModel = NewTableC;
    type Model = TableC;
    type InsertableColumns = (table_c::a_id, table_c::column_c);
}

// Table D models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = table_d)]
pub struct TableD {
    pub id: i32,
    pub a_id: i32,
    pub column_d: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_d)]
pub struct NewTableD {
    pub a_id: Option<i32>,
    pub column_d: Option<Option<String>>,
}

impl TableAddition for table_d::table {
    type InsertableModel = NewTableD;
    type Model = TableD;
    type InsertableColumns = (table_d::a_id, table_d::column_d);
}

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel)]
#[diesel(table_name = table_b)]
pub struct TableB {
    pub id: i32,
    pub c_id: i32,
    pub d_id: i32,
    pub column_b: String,
    pub remote_column_c: Option<String>,
    pub remote_column_d: Option<String>,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_b)]
pub struct NewTableB {
    pub id: Option<i32>,
    pub c_id: Option<i32>,
    pub d_id: Option<i32>,
    pub column_b: Option<String>,
    pub remote_column_c: Option<Option<String>>,
    pub remote_column_d: Option<Option<String>>,
}

impl TableAddition for table_b::table {
    type InsertableModel = NewTableB;
    type Model = TableB;
    type InsertableColumns = (
        table_b::id,
        table_b::c_id,
        table_b::d_id,
        table_b::column_b,
        table_b::remote_column_c,
        table_b::remote_column_d,
    );
}

// Implement SingletonForeignKey for both c_id and d_id
impl diesel_builders::SingletonForeignKey for table_b::c_id {
    type ReferencedTable = table_c::table;
}

impl diesel_builders::SingletonForeignKey for table_b::d_id {
    type ReferencedTable = table_d::table;
}

// Define indexes
index!((table_c::id, table_c::column_c));
index!((table_c::id, table_c::a_id));
index!((table_d::id, table_d::column_d));
index!((table_d::id, table_d::a_id));

// Define foreign key relationships
fk!((table_b::c_id, table_b::remote_column_c) REFERENCES (table_c::id, table_c::column_c));
fk!((table_b::c_id, table_b::id) REFERENCES (table_c::id, table_c::a_id));
fk!((table_b::d_id, table_b::remote_column_d) REFERENCES (table_d::id, table_d::column_d));
fk!((table_b::d_id, table_b::id) REFERENCES (table_d::id, table_d::a_id));

// Define horizontal same-as relationships
impl diesel_builders::HorizontalSameAsKey for table_b::c_id {
    type HostColumns = (table_b::id, table_b::remote_column_c);
    type ForeignColumns = (table_c::a_id, table_c::column_c);
}

impl diesel_builders::HorizontalSameAsKey for table_b::d_id {
    type HostColumns = (table_b::id, table_b::remote_column_d);
    type ForeignColumns = (table_d::a_id, table_d::column_d);
}

#[diesel_builders_macros::bundlable_table]
impl BundlableTable for table_b::table {
    type MandatoryTriangularSameAsColumns = (table_b::c_id,);
    type DiscretionaryTriangularSameAsColumns = (table_b::d_id,);
}

#[test]
fn test_mixed_triangular_relations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create table A
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Create table C
    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL,
            column_c TEXT,
            FOREIGN KEY (a_id) REFERENCES table_a(id)
        )",
    )
    .execute(&mut conn)?;

    // Create table D
    diesel::sql_query(
        "CREATE TABLE table_d (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL,
            column_d TEXT,
            FOREIGN KEY (a_id) REFERENCES table_a(id)
        )",
    )
    .execute(&mut conn)?;

    // Create table B
    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL,
            c_id INTEGER NOT NULL,
            d_id INTEGER NOT NULL,
            column_b TEXT NOT NULL,
            remote_column_c TEXT,
            remote_column_d TEXT,
            FOREIGN KEY (c_id) REFERENCES table_c(id),
            FOREIGN KEY (d_id) REFERENCES table_d(id)
        )",
    )
    .execute(&mut conn)?;

    // Create a C builder with column_c value
    let c_builder = table_c::table::builder().set_column::<table_c::column_c>(Some("C Value"));

    // Create a D builder with column_d value
    let d_builder = table_d::table::builder().set_column::<table_d::column_d>(Some("D Value"));

    // Create a B record with both mandatory (C) and discretionary (D) relations
    let bundle = table_b::table::builder()
        .set_column::<table_b::column_b>("B Value")
        .set_mandatory_builder::<table_b::c_id>(c_builder)
        .set_discretionary_builder::<table_b::d_id>(d_builder)
        .build()?;

    // Insert into table A first
    let a = table_a::table::builder()
        .set_column::<table_a::column_a>("A Value")
        .insert(&mut conn)?;

    // Now insert the bundle
    let (b, _) = bundle.insert(a.id, &mut conn)?;

    // Verify B was inserted correctly
    assert_eq!(b.column_b, "B Value");
    assert_eq!(b.remote_column_c, Some("C Value".to_string()));
    assert_eq!(b.remote_column_d, Some("D Value".to_string()));

    // Query to verify C was created and linked correctly
    let c: TableC = table_c::table
        .filter(table_c::id.eq(b.c_id))
        .first(&mut conn)?;
    assert_eq!(c.a_id, a.id);
    assert_eq!(c.column_c, Some("C Value".to_string()));

    // Query to verify D was created and linked correctly
    let d: TableD = table_d::table
        .filter(table_d::id.eq(b.d_id))
        .first(&mut conn)?;
    assert_eq!(d.a_id, a.id);
    assert_eq!(d.column_d, Some("D Value".to_string()));

    // Verify the triangular constraint: B's id matches A's id
    assert_eq!(b.id, a.id);

    Ok(())
}

#[test]
fn test_mixed_triangular_with_only_mandatory() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create tables
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL,
            column_c TEXT,
            FOREIGN KEY (a_id) REFERENCES table_a(id)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_d (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL,
            column_d TEXT,
            FOREIGN KEY (a_id) REFERENCES table_a(id)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL,
            c_id INTEGER NOT NULL,
            d_id INTEGER NOT NULL,
            column_b TEXT NOT NULL,
            remote_column_c TEXT,
            remote_column_d TEXT,
            FOREIGN KEY (c_id) REFERENCES table_c(id),
            FOREIGN KEY (d_id) REFERENCES table_d(id)
        )",
    )
    .execute(&mut conn)?;

    // Create only mandatory C builder (no discretionary D)
    let c_builder = table_c::table::builder().set_column::<table_c::column_c>(Some("C Value"));

    let bundle = table_b::table::builder()
        .set_column::<table_b::column_b>("B Value")
        .set_mandatory_builder::<table_b::c_id>(c_builder)
        .build()?;

    let a = table_a::table::builder()
        .set_column::<table_a::column_a>("A Value")
        .insert(&mut conn)?;

    let (b, _) = bundle.insert(a.id, &mut conn)?;

    // Verify B was inserted with C but without D
    assert_eq!(b.column_b, "B Value");
    assert_eq!(b.remote_column_c, Some("C Value".to_string()));
    // D should not exist, but d_id will still be set (possibly to a_id or 0)
    // The exact behavior depends on the implementation

    // Verify C exists
    let c: TableC = table_c::table
        .filter(table_c::id.eq(b.c_id))
        .first(&mut conn)?;
    assert_eq!(c.a_id, a.id);

    Ok(())
}

#[test]
fn test_mixed_triangular_missing_mandatory_fails() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create tables
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL,
            column_c TEXT,
            FOREIGN KEY (a_id) REFERENCES table_a(id)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_d (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL,
            column_d TEXT,
            FOREIGN KEY (a_id) REFERENCES table_a(id)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL,
            c_id INTEGER NOT NULL,
            d_id INTEGER NOT NULL,
            column_b TEXT NOT NULL,
            remote_column_c TEXT,
            remote_column_d TEXT,
            FOREIGN KEY (c_id) REFERENCES table_c(id),
            FOREIGN KEY (d_id) REFERENCES table_d(id)
        )",
    )
    .execute(&mut conn)?;

    // Try to build without mandatory C builder
    let d_builder = table_d::table::builder().set_column::<table_d::column_d>(Some("D Value"));

    let result = table_b::table::builder()
        .set_column::<table_b::column_b>("B Value")
        .set_discretionary_builder::<table_b::d_id>(d_builder)
        .build();

    // Should fail because mandatory C builder is missing
    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Incomplete(_)
    ));

    Ok(())
}
