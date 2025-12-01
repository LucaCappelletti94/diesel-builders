//! Example: Inheritance Chain
//!
//! This example demonstrates linear inheritance where tables form a chain:
//! A → B → C. The builder automatically determines the insertion order.
//!
//! Run with: `cargo run --example inheritance_chain`

use diesel_builders::prelude::*;

diesel::table! {
    /// Table A schema (root)
    table_a (id) {
        /// ID
        id -> Integer,
        /// Column A
        column_a -> Text,
    }
}

diesel::table! {
    /// Table B schema (extends A)
    table_b (id) {
        /// ID
        id -> Integer,
        /// Column B
        column_b -> Text,
    }
}

diesel::table! {
    /// Table C schema (extends B)
    table_c (id) {
        /// ID
        id -> Integer,
        /// Column C
        column_c -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(table_a, table_b, table_c);

// Table A (Root)
/// Table A model
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, Root, TableModel)]
#[diesel(table_name = table_a)]
pub struct TableA {
    /// ID
    pub id: i32,
    /// Column A
    pub column_a: String,
}

/// New Table A builder
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_a)]
pub struct NewTableA {
    /// Column A
    pub column_a: Option<String>,
}

impl TableAddition for table_a::table {
    type InsertableModel = NewTableA;
    type Model = TableA;
    type InsertableColumns = (table_a::column_a,);
}

// Table B (extends A)
/// Table B model
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, TableModel, Decoupled)]
#[diesel(table_name = table_b)]
pub struct TableB {
    /// ID
    pub id: i32,
    /// Column B
    pub column_b: String,
}

#[descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

/// New Table B builder
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_b)]
pub struct NewTableB {
    /// ID
    pub id: Option<i32>,
    /// Column B
    pub column_b: Option<String>,
}

impl TableAddition for table_b::table {
    type InsertableModel = NewTableB;
    type Model = TableB;
    type InsertableColumns = (table_b::id, table_b::column_b);
}

// Table C (extends B, which extends A)
/// Table C model
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, TableModel, Decoupled)]
#[diesel(table_name = table_c)]
pub struct TableC {
    /// ID
    pub id: i32,
    /// Column C
    pub column_c: String,
}

#[descendant_of]
impl Descendant for table_c::table {
    type Ancestors = (table_a::table, table_b::table);
    type Root = table_a::table;
}

/// New Table C builder
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_c)]
pub struct NewTableC {
    /// ID
    pub id: Option<i32>,
    /// Column C
    pub column_c: Option<String>,
}

impl TableAddition for table_c::table {
    type InsertableModel = NewTableC;
    type Model = TableC;
    type InsertableColumns = (table_c::id, table_c::column_c);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Create tables
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_a(id),
            column_b TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_b(id),
            column_c TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Insert into table C (which extends B, which extends A)
    // The builder automatically handles the insertion order: A → B → C
    let c: TableC = table_c::table::builder()
        .set_column::<table_a::column_a>(&"Value A".to_string())
        .set_column::<table_b::column_b>(&"Value B".to_string())
        .set_column::<table_c::column_c>(&"Value C".to_string())
        .insert(&mut conn)?;

    println!("Successfully inserted TableC: {c:?}");
    assert_eq!(c.column_c, "Value C");

    println!("\nInheritance chain insertion completed successfully!");
    Ok(())
}
