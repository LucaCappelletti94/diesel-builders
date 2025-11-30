//! Test case for foreign key based inheritance where the dependencies
//! form a chain, with A being the root, B extending A, and C extending B.

use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_additions::{SetColumnExt, TableAddition};
use diesel_builders::{BuildableTable, BundlableTable, NestedInsert};
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

// Define table C (extends B)
diesel::table! {
    /// Table C extends B via foreign key.
    table_c (id) {
        /// Primary key of table C, foreign key to table_b.id.
        id -> Integer,
        /// A simple column for table C.
        column_c -> Text,
    }
}

// Define join relationships
diesel::joinable!(table_b -> table_a (id));
diesel::joinable!(table_c -> table_b (id));

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

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel)]
#[diesel(table_name = table_b)]
/// Model for table B.
pub struct TableB {
    /// Primary key.
    pub id: i32,
    /// Column B value.
    pub column_b: String,
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
    /// Column B value.
    pub column_b: Option<String>,
}

impl TableAddition for table_b::table {
    type InsertableModel = NewTableB;
    type Model = TableB;
    type InsertableColumns = (table_b::id, table_b::column_b);
}

impl BundlableTable for table_b::table {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = ();
}

// Table C models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel)]
#[diesel(table_name = table_c)]
/// Model for table C.
pub struct TableC {
    /// Primary key.
    pub id: i32,
    /// Column C value.
    pub column_c: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_c::table {
    type Ancestors = (table_a::table, table_b::table);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_c)]
/// Insertable model for table C.
pub struct NewTableC {
    /// Primary key.
    pub id: Option<i32>,
    /// Column C value.
    pub column_c: Option<String>,
}

impl TableAddition for table_c::table {
    type InsertableModel = NewTableC;
    type Model = TableC;
    type InsertableColumns = (table_c::id, table_c::column_c);
}

impl BundlableTable for table_c::table {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = ();
}

#[test]
fn test_inheritance_chain() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

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

    // Create table C (extends B)
    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_b(id),
            column_c TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let a = table_a::table::builder()
        .set_column::<table_a::column_a>(&"Value A".to_string())
        .insert(&mut conn)?;

    assert_eq!(a.column_a, "Value A");

    // Insert into table B (extends A)
    let b = table_b::table::builder()
        .set_column::<table_a::column_a>(&"Value A for B".to_string())
        .set_column::<table_b::column_b>(&"Value B".to_string())
        .insert(&mut conn)?;

    assert_eq!(b.column_b, "Value B");

    // Verify B can be queried
    let queried_b: TableB = table_b::table.filter(table_b::id.eq(b.id)).first(&mut conn)?;
    assert_eq!(queried_b, b);

    // Insert into table C (extends B, transitively extends A)
    let c = table_c::table::builder()
        .set_column::<table_a::column_a>(&"Value A for C".to_string())
        .set_column::<table_b::column_b>(&"Value B for C".to_string())
        .set_column::<table_c::column_c>(&"Value C".to_string())
        .insert(&mut conn)?;

    assert_eq!(c.column_c, "Value C");

    // Verify C can be queried
    let queried_c: TableC = table_c::table.filter(table_c::id.eq(c.id)).first(&mut conn)?;
    assert_eq!(queried_c, c);

    // Verify we can join through the chain: A -> B
    let (loaded_a, loaded_b): (TableA, TableB) =
        table_a::table.inner_join(table_b::table).filter(table_b::id.eq(b.id)).first(&mut conn)?;

    assert_eq!(loaded_a.id, loaded_b.id);
    assert_eq!(loaded_b, b);

    // Verify we can join through the chain: B -> C
    let (loaded_b2, loaded_c): (TableB, TableC) =
        table_b::table.inner_join(table_c::table).filter(table_c::id.eq(c.id)).first(&mut conn)?;

    assert_eq!(loaded_b2.id, loaded_c.id);
    assert_eq!(loaded_c, c);

    // Verify we can join through the full chain: A -> B -> C
    let (loaded_a2, (loaded_b3, loaded_c2)): (TableA, (TableB, TableC)) = table_a::table
        .inner_join(table_b::table.inner_join(table_c::table))
        .filter(table_c::id.eq(c.id))
        .first(&mut conn)?;

    assert_eq!(loaded_a2.id, loaded_b3.id);
    assert_eq!(loaded_b3.id, loaded_c2.id);
    assert_eq!(loaded_c2, c);

    Ok(())
}
