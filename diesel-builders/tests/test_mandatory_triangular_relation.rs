//! Submodule to test a mandatory triangular relation between tables.
//!
//! This test sets up three tables: A, B, and C. B extends C, and C contains
//! a column that references A, and B has a column that references C, forming a
//! triangular relationship. The test verifies that inserts and queries work
//! correctly through this relationship.
//!
//! Specifically, the relationship is mandatory, that is the foreign key from
//! C to A is referenced in B using a same-as relationship, which means that
//! the C record associated with a B record must reference the same A record as
//! B does. Furthermore, another column in B is linked via the same-as
//! relationship to a column in C, value which needs to be set when setting the
//! builder for the C record in the B builder.

mod common;

use std::convert::Infallible;

use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel_builders::{
    CompletedTableBuilderBundle, SetColumn, TableBuilder, TableBuilderBundle, TrySetColumn,
    prelude::*, table_builder::CompletedTableBuilder,
};
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel};

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
        column_c -> Nullable<Text>,
    }
}

// Define table B (extends C and has its own foreign key to A)
diesel::table! {
    /// Table B extends C via foreign key and also references A.
    table_b (id) {
        /// Primary key of table B, foreign key to table_c.id.
        id -> Integer,
        /// Foreign key to table C (must match C's a_id - mandatory triangular relation).
        c_id -> Integer,
        /// A simple column for table B.
        column_b -> Text,
        /// The remote column_c value from table C that B references via c_id.
        remote_column_c -> Nullable<Text>,
    }
}

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
    pub column_c: Option<String>,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = table_c)]
/// Insertable model for table C.
pub struct NewTableC {
    /// Foreign key to table A.
    pub a_id: Option<i32>,
    /// Column C value.
    pub column_c: Option<Option<String>>,
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
    pub remote_column_c: Option<String>,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Clone, PartialEq)]
/// Errors for NewTableB validation.
pub enum ErrorB {
    /// remote_column_c cannot be empty.
    EmptyRemoteColumnC,
}

impl std::fmt::Display for ErrorB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorB::EmptyRemoteColumnC => write!(f, "remote_column_c cannot be empty"),
        }
    }
}

impl From<Infallible> for ErrorB {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl core::error::Error for ErrorB {}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, HasTable)]
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
    pub remote_column_c: Option<Option<String>>,
}

impl diesel_builders::MaySetColumn<table_b::id> for NewTableB {
    fn may_set_column(&mut self, value: Option<&i32>) -> &mut Self {
        if let Some(v) = value {
            self.id = Some(*v);
        }
        self
    }
}

impl SetColumn<table_b::id> for NewTableB {
    fn set_column(&mut self, value: impl Into<i32>) -> &mut Self {
        self.id = Some(value.into());
        self
    }
}

impl TrySetColumn<table_b::id> for NewTableB {
    type Error = std::convert::Infallible;

    fn try_set_column(&mut self, value: i32) -> Result<&mut Self, Self::Error> {
        self.id = Some(value);
        Ok(self)
    }
}

impl diesel_builders::MaySetColumn<table_b::c_id> for NewTableB {
    fn may_set_column(&mut self, value: Option<&i32>) -> &mut Self {
        if let Some(v) = value {
            self.c_id = Some(*v);
        }
        self
    }
}

impl SetColumn<table_b::c_id> for NewTableB {
    fn set_column(&mut self, value: impl Into<i32>) -> &mut Self {
        self.c_id = Some(value.into());
        self
    }
}

impl TrySetColumn<table_b::c_id> for NewTableB {
    type Error = std::convert::Infallible;

    fn try_set_column(&mut self, value: i32) -> Result<&mut Self, Self::Error> {
        self.c_id = Some(value);
        Ok(self)
    }
}

impl diesel_builders::MaySetColumn<table_b::column_b> for NewTableB {
    fn may_set_column(&mut self, value: Option<&String>) -> &mut Self {
        if let Some(v) = value {
            self.column_b = Some(v.clone());
        }
        self
    }
}

impl SetColumn<table_b::column_b> for NewTableB {
    fn set_column(&mut self, value: impl Into<String>) -> &mut Self {
        self.column_b = Some(value.into());
        self
    }
}

impl TrySetColumn<table_b::column_b> for NewTableB {
    type Error = std::convert::Infallible;

    fn try_set_column(&mut self, value: String) -> Result<&mut Self, Self::Error> {
        self.column_b = Some(value);
        Ok(self)
    }
}

impl diesel_builders::MaySetColumn<table_b::remote_column_c> for NewTableB {
    fn may_set_column(&mut self, value: Option<&Option<String>>) -> &mut Self {
        if let Some(v) = value {
            self.remote_column_c = Some(v.clone());
        }
        self
    }
}

impl TrySetColumn<table_b::remote_column_c> for NewTableB {
    type Error = ErrorB;

    fn try_set_column(&mut self, value: Option<String>) -> Result<&mut Self, Self::Error> {
        if let Some(ref v) = value
            && v.trim().is_empty()
        {
            return Err(ErrorB::EmptyRemoteColumnC);
        }
        self.remote_column_c = Some(value);
        Ok(self)
    }
}

impl InsertableTableModel for NewTableB {
    type Error = ErrorB;
}

impl TableAddition for table_b::table {
    type InsertableModel = NewTableB;
    type Model = TableB;
    type InsertableColumns = (
        table_b::id,
        table_b::c_id,
        table_b::column_b,
        table_b::remote_column_c,
    );
}

// Implement SingletonForeignKey for table_b::c_id to indicate it references
// table_c
impl diesel_builders::SingletonForeignKey for table_b::c_id {
    type ReferencedTable = table_c::table;
}

// Define table indices that can be referenced by foreign keys
index!((table_c::id, table_c::a_id));
index!((table_c::id, table_c::column_c));

// Define foreign key relationships using SQL-like syntax
// B's (c_id, id) references C's (id, a_id) - ensures triangular consistency
fk!((table_b::c_id, table_b::id) REFERENCES (table_c::id, table_c::a_id));

// B's (c_id, remote_column_c) references C's (id, column_c)
fk!((table_b::c_id, table_b::remote_column_c) REFERENCES (table_c::id, table_c::column_c));

// This is the key part: B's c_id must match C's id, and C's a_id must match A's
// id. We express that B's c_id is horizontally the same as C's a_id, which in
// turn is the same as A's id.
impl diesel_builders::HorizontalSameAsKey for table_b::c_id {
    // HostColumns are columns in table_b (the same table) that relate to this key
    // In this case, there are no other columns in table_b that need to match
    // Actually, we need to think about this differently...
    type HostColumns = (table_b::id, table_b::remote_column_c);
    type ForeignColumns = (table_c::a_id, table_c::column_c);
}

#[diesel_builders_macros::bundlable_table]
impl BundlableTable for table_b::table {
    type MandatoryTriangularSameAsColumns = (table_b::c_id,);
    type DiscretionaryTriangularSameAsColumns = ();
}

#[test]
fn test_mandatory_triangular_relation() -> Result<(), Box<dyn std::error::Error>> {
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
            column_c TEXT,
			UNIQUE(id, a_id),
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
            remote_column_c TEXT CHECK (remote_column_c <> ''),
			FOREIGN KEY (c_id, id) REFERENCES table_c(id, a_id),
            FOREIGN KEY (c_id, remote_column_c) REFERENCES table_c(id, column_c)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let a = table_a::table::builder()
        .set_column::<table_a::column_a>("Value A")
        .insert(&mut conn)?;

    assert_eq!(a.column_a, "Value A");

    // Insert into table C (references A)
    let c = table_c::table::builder()
        .set_column::<table_c::a_id>(a.id)
        .set_column::<table_c::column_c>(Some("Value C".to_owned()))
        .insert(&mut conn)?;

    assert_eq!(c.column_c.as_deref(), Some("Value C"));
    assert_eq!(c.a_id, a.id);

    let mut c_builder = table_c::table::builder();
    c_builder.set_column_ref::<table_c::column_c>(Some("Value C".to_owned()));

    // Insert into table B (extends C and references A)
    // The mandatory triangular relation means B's a_id should automatically
    // match C's a_id when we only set C's columns
    let mut b_builder = table_b::table::builder();

    assert!(matches!(
        b_builder.try_set_mandatory_builder_ref::<table_b::c_id>(
            table_c::table::builder().set_column::<table_c::column_c>(String::new())
        ),
        Err(ErrorB::EmptyRemoteColumnC)
    ));

    b_builder
        .set_column_ref::<table_a::column_a>("Value A for B")
        .set_column_ref::<table_b::column_b>("Value B")
        .try_set_mandatory_builder_ref::<table_b::c_id>(c_builder.clone())?;

    let b = b_builder
        .try_set_mandatory_builder::<table_b::c_id>(c_builder)?
        .insert(&mut conn)?;

    let associated_a: TableA = table_a::table
        .filter(table_a::id.eq(b.id))
        .first(&mut conn)
        .unwrap();
    assert_eq!(associated_a.column_a, "Value A for B");

    let associated_c: TableC = table_c::table
        .filter(table_c::id.eq(b.c_id))
        .first(&mut conn)
        .unwrap();
    assert_eq!(associated_c.column_c.as_deref(), Some("Value C"));
    assert_eq!(associated_c.a_id, b.id);
    assert_eq!(associated_c.a_id, associated_a.id);

    let _ = TableBuilderBundle::<table_b::table>::table();
    let _ = CompletedTableBuilderBundle::<table_b::table>::table();
    let _ = TableBuilder::<table_b::table>::table();
    let _ = CompletedTableBuilder::<table_b::table, ()>::table();

    Ok(())
}

#[test]
fn test_mandatory_triangular_relation_missing_builder_error() {
    use diesel_builders::{CompletedTableBuilderBundle, TableBuilderBundle};
    use std::convert::TryFrom;

    // Create a TableBuilderBundle without setting the mandatory associated builder
    let b_bundle = TableBuilderBundle::<table_b::table>::default();

    // Try to convert to CompletedTableBuilderBundle - this should fail because
    // the mandatory associated builder for c_id has not been set
    let result = CompletedTableBuilderBundle::try_from(b_bundle);

    // Verify that the conversion fails with the expected error message
    assert!(result.is_err());
    if let Err(error) = result {
        assert_eq!(
            error.to_string(),
            "Not all mandatory associated builders have been set"
        );
    }
}

#[test]
fn test_mandatory_triangular_insert_fails_when_c_table_missing()
-> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create table A
    diesel::sql_query(
        "CREATE TABLE table_a (
            id INTEGER PRIMARY KEY NOT NULL,
            column_a TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Intentionally do NOT create table_c

    // Create table B (which references C)
    diesel::sql_query(
        "CREATE TABLE table_b (
            id INTEGER PRIMARY KEY NOT NULL,
            c_id INTEGER NOT NULL,
            column_b TEXT NOT NULL,
            remote_column_c TEXT
        )",
    )
    .execute(&mut conn)?;

    // Try to insert into B with a mandatory C builder
    let c_builder = table_c::table::builder().set_column::<table_c::column_c>(None);

    let result = table_b::table::builder()
        .set_column::<table_b::column_b>("B Value")
        .set_mandatory_builder::<table_b::c_id>(c_builder)
        .insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Diesel(_)
    ));

    Ok(())
}
