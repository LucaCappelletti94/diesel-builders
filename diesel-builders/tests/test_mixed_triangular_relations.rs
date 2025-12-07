//! Test case for a table with both mandatory and discretionary triangular relations.
//!
//! This test sets up four tables: A (root), C (references A), D (references A),
//! and B which:
//! - Has a mandatory triangular relation with C (via c_id)
//! - Has a discretionary triangular relation with D (via d_id)

mod common;

use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel};

diesel::table! {
    /// Root table A.
    table_a (id) {
        /// Primary key of table A.
        id -> Integer,
        /// Column A value.
        column_a -> Text,
    }
}

diesel::table! {
    /// Table C with a foreign key to A.
    table_c (id) {
        /// Primary key of table C.
        id -> Integer,
        /// Foreign key to table A.
        a_id -> Integer,
        /// Column C value.
        column_c -> Nullable<Text>,
    }
}

diesel::table! {
    /// Table D with a foreign key to A.
    table_d (id) {
        /// Primary key of table D.
        id -> Integer,
        /// Foreign key to table A.
        a_id -> Integer,
        /// Column D value.
        column_d -> Nullable<Text>,
    }
}

diesel::table! {
    /// Table B with both mandatory and discretionary triangular relations.
    table_b (id) {
        /// Primary key of table B.
        id -> Integer,
        /// Foreign key to table C (mandatory triangular relation).
        c_id -> Integer,
        /// Foreign key to table D (discretionary triangular relation).
        d_id -> Integer,
        /// Column B value.
        column_b -> Text,
        /// The remote column_c value from table C.
        remote_column_c -> Nullable<Text>,
        /// The remote column_d value from table D.
        remote_column_d -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(table_a, table_b, table_c, table_d);

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = table_a)]
/// Insertable model for table A.
pub struct NewTableA {
    /// Column A value.
    pub column_a: Option<String>,
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
    pub column_c: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = table_c)]
/// Insertable model for table C.
pub struct NewTableC {
    /// Foreign key to table A.
    pub a_id: Option<i32>,
    /// Column C value.
    pub column_c: Option<Option<String>>,
}

// Table D models
#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = table_d)]
/// Model for table D.
pub struct TableD {
    /// Primary key.
    pub id: i32,
    /// Foreign key to table A.
    pub a_id: i32,
    /// Column D value.
    pub column_d: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = table_d)]
/// Insertable model for table D.
pub struct NewTableD {
    /// Foreign key to table A.
    pub a_id: Option<i32>,
    /// Column D value.
    pub column_d: Option<Option<String>>,
}

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, TableModel)]
#[diesel(table_name = table_b)]
/// Model for table B.
pub struct TableB {
    /// Primary key.
    pub id: i32,
    /// Foreign key to table C.
    pub c_id: i32,
    /// Foreign key to table D.
    pub d_id: i32,
    /// Column B value.
    pub column_b: String,
    /// Remote column C value.
    pub remote_column_c: Option<String>,
    /// Remote column D value.
    pub remote_column_d: Option<String>,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = table_b)]
/// Insertable model for table B.
pub struct NewTableB {
    /// Primary key.
    pub id: Option<i32>,
    /// Foreign key to table C.
    pub c_id: Option<i32>,
    /// Foreign key to table D.
    pub d_id: Option<i32>,
    /// Column B value.
    pub column_b: Option<String>,
    /// Remote column C value.
    pub remote_column_c: Option<Option<String>>,
    /// Remote column D value.
    pub remote_column_d: Option<Option<String>>,
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

fn create_tables(conn: &mut SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE TABLE table_c (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL REFERENCES table_a(id),
            column_c TEXT,
            UNIQUE(id, a_id),
            UNIQUE(id, column_c)
        )",
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE TABLE table_d (
            id INTEGER PRIMARY KEY NOT NULL,
            a_id INTEGER NOT NULL REFERENCES table_a(id),
            column_d TEXT,
            UNIQUE(id, a_id),
            UNIQUE(id, column_d)
        )",
    )
    .execute(conn)?;

    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES table_a(id),
            c_id INTEGER NOT NULL REFERENCES table_c(id),
            d_id INTEGER NOT NULL REFERENCES table_d(id),
            column_b TEXT NOT NULL,
            remote_column_c TEXT,
            remote_column_d TEXT,
            FOREIGN KEY (c_id, id) REFERENCES table_c(id, a_id),
            FOREIGN KEY (c_id, remote_column_c) REFERENCES table_c(id, column_c),
            FOREIGN KEY (d_id, id) REFERENCES table_d(id, a_id),
            FOREIGN KEY (d_id, remote_column_d) REFERENCES table_d(id, column_d)
        )",
    )
    .execute(conn)?;

    Ok(())
}

#[test]
fn test_mixed_triangular_relations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;
    create_tables(&mut conn)?;

    // Insert B with both mandatory C and discretionary D
    // Using generated trait methods for ergonomic builder setup
    let b = table_b::table::builder()
        .column_a("Value A for B")
        .column_b("Value B")
        .c_id_builder(table_c::table::builder().column_c("Value C".to_owned()))
        .d_id_builder(table_d::table::builder().column_d("Value D".to_owned()))
        .insert(&mut conn)?;

    assert_eq!(b.column_b, "Value B");
    assert_eq!(b.remote_column_c.as_deref(), Some("Value C"));
    assert_eq!(b.remote_column_d.as_deref(), Some("Value D"));

    // Verify associated C
    let c: TableC = table_c::table
        .filter(table_c::id.eq(b.c_id))
        .first(&mut conn)?;
    assert_eq!(c.a_id, b.id);
    assert_eq!(c.column_c.as_deref(), Some("Value C"));

    // Verify associated D
    let d: TableD = table_d::table
        .filter(table_d::id.eq(b.d_id))
        .first(&mut conn)?;
    assert_eq!(d.a_id, b.id);
    assert_eq!(d.column_d.as_deref(), Some("Value D"));

    Ok(())
}

#[test]
fn test_mixed_triangular_missing_mandatory_fails() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;
    create_tables(&mut conn)?;

    let table_a = table_a::table::builder()
        .column_a("Value A")
        .insert(&mut conn)?;

    let table_d = table_d::table::builder()
        .a_id(table_a.id)
        .column_d("Value D".to_owned())
        .insert(&mut conn)?;

    // Try to create without mandatory C builder
    // Note: d_id_model references an existing model instead of creating a new one
    let result = table_b::table::builder()
        .column_a("Value A")
        .column_b("Value B")
        .d_id_builder(table_d::table::builder().column_d("Value D".to_owned()))
        .d_id_model(&table_d)
        .insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Incomplete(_)
    ));

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder with mixed mandatory and discretionary triangular relations
    let builder = table_b::table::builder()
        .column_b("Serialized B")
        .try_remote_column_c(Some("Serialized C".to_string()))?
        .try_remote_column_d(Some("Serialized D".to_string()))?;

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<table_b::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(
        deserialized
            .may_get_column_ref::<table_b::column_b>()
            .map(String::as_str),
        Some("Serialized B")
    );
    assert_eq!(
        deserialized.may_get_column_ref::<table_b::remote_column_c>(),
        Some(&Some("Serialized C".to_string()))
    );
    assert_eq!(
        deserialized.may_get_column_ref::<table_b::remote_column_d>(),
        Some(&Some("Serialized D".to_string()))
    );

    Ok(())
}
