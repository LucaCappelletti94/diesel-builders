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

use std::convert::Infallible;

use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::{Root, TableModel};

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
        /// Primary key of table B, foreign key to `table_c.id`.
        id -> Integer,
        /// Foreign key to table C (must match C's `a_id` - discretionary triangular relation).
        c_id -> Integer,
        /// A simple column for table B.
        column_b -> Text,
        /// The remote `column_c` value from table C that B references via `c_id`.
        remote_column_c -> Nullable<Text>,
    }
}

// Allow tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(table_a, table_b, table_c);

// Table A models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, Root, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = table_a)]
/// Model for table A.
pub struct TableA {
    /// Primary key.
    id: i32,
    /// Column A value.
    column_a: String,
}

// Table C models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, Root, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = table_c)]
/// Model for table C.
pub struct TableC {
    /// Primary key.
    id: i32,
    /// Foreign key to table A.
    a_id: i32,
    /// Column C value.
    column_c: Option<String>,
}

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(error = ErrorB)]
#[diesel(table_name = table_b)]
/// Model for table B.
pub struct TableB {
    #[infallible]
    /// Primary key.
    id: i32,
    #[infallible]
    /// Foreign key to table A.
    c_id: i32,
    #[infallible]
    /// Column B value.
    column_b: String,
    /// The remote `column_c` value from table C that B references via `c_id`.
    remote_column_c: Option<String>,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for table_b::table {
    type Ancestors = (table_a::table,);
    type Root = table_a::table;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
/// Errors for `NewTableB` validation.
pub enum ErrorB {
    /// `remote_column_c` cannot be empty.
    #[error("`remote_column_c` cannot be empty")]
    EmptyRemoteColumnC,
}

impl From<Infallible> for ErrorB {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl TrySetColumn<table_b::remote_column_c> for <table_b::table as TableExt>::NewValues {
    type Error = ErrorB;

    fn try_set_column(&mut self, value: Option<String>) -> Result<&mut Self, Self::Error> {
        if let Some(ref v) = value
            && v.trim().is_empty()
        {
            return Err(ErrorB::EmptyRemoteColumnC);
        }
        self.set_column_unchecked::<table_b::remote_column_c>(value);
        Ok(self)
    }
}

// Declare singleton foreign key for table_b::c_id to table_c
fpk!(table_b::c_id -> table_c);
// Declare singleton foreign key for table_b::id to table_a (inheritance)
fpk!(table_b::id -> table_a);

// Define table index that can be referenced by foreign keys
index!(table_c::id, table_c::column_c);
index!(table_c::id, table_c::a_id);

// Define foreign key relationship using SQL-like syntax
// B's (c_id, remote_column_c) references C's (id, column_c)
fk!((table_b::c_id, table_b::remote_column_c) -> (table_c::id, table_c::column_c));
fk!((table_b::c_id, table_b::id) -> (table_c::id, table_c::a_id));

// This is the key part: B's c_id must match C's id, and C's a_id must match A's
// id. We express that B's c_id is horizontally the same as C's a_id, which in
// turn is the same as A's id.
impl diesel_builders::HorizontalKey for table_b::c_id {
    // HostColumns are columns in table_b (the same table) that relate to this key
    // In this case, there are no other columns in table_b that need to match
    // Actually, we need to think about this differently...
    type HostColumns = (table_b::id, table_b::remote_column_c);
    type ForeignColumns = (table_c::a_id, table_c::column_c);
}

#[diesel_builders_macros::bundlable_table]
impl BundlableTable for table_b::table {
    type MandatoryTriangularColumns = ();
    type DiscretionaryTriangularColumns = (table_b::c_id,);
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
            column_c TEXT,
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
			FOREIGN KEY (c_id, remote_column_c) REFERENCES table_c(id, column_c)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let a = table_a::table::builder()
        .column_a("Value A")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(a.get_column::<table_a::column_a>(), "Value A");

    // Insert into table C (references A)
    let c = table_c::table::builder()
        .a_id(a.get_column::<table_a::id>())
        .column_c(Some("Value C".to_owned()))
        .insert(&mut conn)
        .unwrap();

    assert_eq!(
        c.get_column::<table_c::column_c>().as_deref(),
        Some("Value C")
    );
    assert_eq!(
        c.get_column::<table_c::a_id>(),
        a.get_column::<table_a::id>()
    );

    let mut c_builder = table_c::table::builder();
    c_builder.column_c_ref(Some("Value C for B".to_owned()));

    // Insert into table B (extends C and references A)
    // The discretionary triangular relation means we can set the C builder or reference an existing C model
    // Using generated trait methods like try_c_ref for type-safe builders
    let mut triangular_b_builder = table_b::table::builder();

    assert!(matches!(
        triangular_b_builder.try_c_ref(table_c::table::builder().column_c(String::new())),
        Err(ErrorB::EmptyRemoteColumnC)
    ));

    triangular_b_builder
        .column_a_ref("Value A for B")
        .column_b_ref("Value B")
        .try_c_ref(c_builder.clone())?;

    // Debug formatting test
    let _formatted = format!("{triangular_b_builder:?}");

    let triangular_b = triangular_b_builder.try_c(c_builder)?.insert(&mut conn)?;

    let associated_a: TableA = triangular_b.id_fk(&mut conn)?;
    assert_eq!(
        associated_a.get_column::<table_a::column_a>(),
        "Value A for B"
    );

    // We can also reference an existing model using the _model variant
    // Example: triangular_b_builder.c_id_model_ref(&c) would reference the existing c model

    let associated_c: TableC = triangular_b.c(&mut conn)?;
    assert_eq!(
        associated_c.get_column::<table_c::column_c>().as_deref(),
        Some("Value C for B")
    );
    assert_eq!(
        associated_c.get_column::<table_c::a_id>(),
        triangular_b.get_column::<table_b::id>()
    );
    assert_eq!(
        associated_c.get_column::<table_c::a_id>(),
        associated_a.get_column::<table_a::id>()
    );

    let indipendent_b = table_b::table::builder()
        .column_a("Independent A for B")
        .column_b("Independent B")
        .try_c_model(&c)?
        .insert(&mut conn)
        .unwrap();

    assert_eq!(
        indipendent_b.get_column::<table_b::column_b>(),
        "Independent B"
    );
    assert_eq!(indipendent_b.remote_column_c.as_deref(), Some("Value C"));
    assert_ne!(
        indipendent_b.get_column::<table_b::id>(),
        triangular_b.get_column::<table_b::id>()
    );
    assert_ne!(
        indipendent_b.get_column::<table_b::id>(),
        triangular_b.get_column::<table_b::id>()
    );
    assert_ne!(
        indipendent_b.get_column::<table_b::id>(),
        c.get_column::<table_c::a_id>()
    );

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder with discretionary triangular relation
    let builder = table_b::table::builder()
        .column_b("Serialized B")
        .try_remote_column_c("Serialized C".to_string())?;

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
        deserialized.may_get_column::<table_b::column_b>(),
        Some("Serialized B".to_owned())
    );
    assert_eq!(
        deserialized.may_get_column_ref::<table_b::remote_column_c>(),
        Some(&Some("Serialized C".to_string()))
    );

    Ok(())
}
