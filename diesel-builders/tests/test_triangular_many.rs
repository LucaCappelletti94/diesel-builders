//! Test for a child table with multiple mandatory and discretionary triangular relations.

mod common;
use diesel_builders::prelude::*;

diesel::table! {
    /// The root parent table for the triangular relations used in this test.
    /// Rows here are referenced by intermediate tables via `a_id`.
    parent_table (id) {
        /// Primary key of the parent table.
        id -> Integer,
        /// A simple text label to identify the parent row.
        name -> Text,
    }
}

diesel::table! {
    /// Mandatory intermediate table; its rows must reference a `parent_table`.
    /// These rows are used for mandatory triangular relations with `child_table`.
    mandatory_table (id) {
        /// Primary key of the intermediate table.
        id -> Integer,
        /// Reference to the `parent_table` row (enforces the ancestor link).
        a_id -> Integer,
        /// Column used for composite foreign keys with `child_table`.
        col -> Text,
    }
}

diesel::table! {
    /// Discretionary intermediate table; `col` is nullable which makes
    /// same-as relationships optional from the child side.
    discretionary_table (id) {
        /// Primary key of discretionary table.
        id -> Integer,
        /// Reference to the `parent_table` row.
        a_id -> Integer,
        /// Optional column used in composite relations; `None` denotes
        /// the column is not set and the relation can be omitted by the
        /// builder.
        col -> Nullable<Text>,
    }
}

diesel::table! {
    /// Child table used to validate multiple triangular relationships.
    /// Each `m*` reference is mandatory, while `d*` are discretionary.
    child_table (id) {
        /// Primary key; also references `parent_table.id` in the SQL DDL.
        id -> Integer,
        /// Mandatory relation 1: intermediate ID.
        m1_id -> Integer,
        /// Mandatory relation 1: composite column used in FKs.
        m1_col -> Text,
        /// Mandatory relation 2: intermediate ID.
        m2_id -> Integer,
        /// Mandatory relation 2: composite column used in FKs.
        m2_col -> Text,
        /// Mandatory relation 3: intermediate ID.
        m3_id -> Integer,
        /// Mandatory relation 3: composite column used in FKs.
        m3_col -> Text,
        /// Discretionary relation 1: intermediate ID.
        d1_id -> Integer,
        /// Discretionary relation 1: optional composite column.
        d1_col -> Nullable<Text>,
        /// Discretionary relation 2: intermediate ID.
        d2_id -> Integer,
        /// Discretionary relation 2: optional composite column.
        d2_col -> Nullable<Text>,
        /// Discretionary relation 3: intermediate ID.
        d3_id -> Integer,
        /// Discretionary relation 3: optional composite column.
        d3_col -> Nullable<Text>,
        /// A simple payload used to verify the insertion in the test.
        payload -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    parent_table,
    mandatory_table,
    discretionary_table,
    child_table,
);

/// Root table used by the test. Intermediate tables and the child table
/// are associated with a `Parent` by `a_id` or `id` FK relationships,
/// depending on the relation type.
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, Root, TableModel)]
#[diesel(table_name = parent_table)]
pub struct Parent {
    id: i32,
    name: String,
}

/// Insertable type used to construct `Parent` rows via the builder API in
/// the test. Fields are `Option` to allow the builder to set them in a
/// fluent, partial manner.
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = parent_table)]
pub struct NewParent {
    name: Option<String>,
}

/// An intermediate table representing a mandatory triangular relation.
/// Each `Mandatory` row is associated with exactly one `Parent` via `a_id`.
/// The `col` column participates in composite FKs with `child_table`.
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel, Root)]
#[diesel(table_name = mandatory_table)]
pub struct Mandatory {
    id: i32,
    a_id: i32,
    col: String,
}

/// Insertable form for `Mandatory` used by the test builder. The `a_id` is
/// optional in the insertable to support nested builders that set it via the
/// parent reference.
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = mandatory_table)]
pub struct NewMandatory {
    a_id: Option<i32>,
    col: Option<String>,
}

// Discretionary intermediate d1
/// An intermediate table used for discretionary triangular relations.
/// Unlike `Mandatory`, the `col` here is nullable, so a child may omit the
/// column and the relation becomes optional from the child side.
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel, Root)]
#[diesel(table_name = discretionary_table)]
pub struct Discretionary {
    id: i32,
    a_id: i32,
    col: Option<String>,
}

/// Insertable variant for `Discretionary` used by the tests. The `col`
/// field is `Option<Option<String>>` to represent a nullable database column
/// while staying builder-friendly.
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = discretionary_table)]
#[allow(clippy::option_option)]
pub struct NewDiscretionary {
    a_id: Option<i32>,
    col: Option<Option<String>>,
}

// Implement Descendant and BundlableTable for `child_table`. These trait
// annotations ensure builder macros derive the correct nested behavior for
// the child, which relies on the parent as its ancestor.
#[diesel_builders_macros::descendant_of]
/// Marker `Descendant` impl: the `child_table` is a descendant of
/// `parent_table` and that relationship is used by the builder to derive
/// ancestry and root-based behavior.
impl Descendant for child_table::table {
    type Ancestors = (parent_table::table,);
    type Root = parent_table::table;
}

/// Declare which child columns correspond to mandatory vs discretionary
/// triangular relations. Bundling relies on these tuples to drive the
/// builder's nested/optional behavior when inserting related records.
#[diesel_builders_macros::bundlable_table]
/// Bundlable trait implementation describing which columns are mandatory and
/// which are discretionary for the test. The builder uses this to enforce
/// and generate the correct nested insert semantics.
impl BundlableTable for child_table::table {
    type MandatoryTriangularColumns = (child_table::m1_id, child_table::m2_id, child_table::m3_id);
    type DiscretionaryTriangularColumns =
        (child_table::d1_id, child_table::d2_id, child_table::d3_id);
}

/// The `Child` table ties together multiple intermediate rows using
/// composite foreign keys. The `payload` field verifies the builder's
/// insertion logic while several `m*` and `d*` columns test mandatory and
/// discretionary triangular relationships.
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = child_table)]
pub struct Child {
    /// The primary key of the child, also a foreign key to `parent_table`.
    id: i32,
    m1_id: i32,
    /// Secondary column used by composite FKs for `Mandatory` intermediates.
    m1_col: String,
    m2_id: i32,
    m2_col: String,
    m3_id: i32,
    m3_col: String,
    d1_id: i32,
    /// Optional secondary column used by composite FKs for discretionary
    /// intermediates; `None` denotes an unset optional value.
    d1_col: Option<String>,
    d2_id: i32,
    d2_col: Option<String>,
    d3_id: i32,
    d3_col: Option<String>,
    /// The payload establishes a simple observable column for test
    /// assertions after insert.
    payload: String,
}

/// Insertable form for `Child` designed for use with the table builder.
/// Each optional field corresponds to a builder-settable column.
#[derive(Default, Debug, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = child_table)]
#[allow(clippy::option_option)]
pub struct NewChild {
    id: Option<i32>,
    m1_id: Option<i32>,
    m1_col: Option<String>,
    m2_id: Option<i32>,
    m2_col: Option<String>,
    m3_id: Option<i32>,
    m3_col: Option<String>,
    d1_id: Option<i32>,
    d1_col: Option<Option<String>>,
    d2_id: Option<i32>,
    d2_col: Option<Option<String>>,
    d3_id: Option<i32>,
    d3_col: Option<Option<String>>,
    payload: Option<String>,
}

// Declare singleton foreign keys for mandatory relationships
fpk!(child_table::m1_id -> mandatory_table);
fpk!(child_table::m2_id -> mandatory_table);
fpk!(child_table::m3_id -> mandatory_table);

// Declare singleton foreign key for child id referencing parent
fpk!(child_table::id -> parent_table);

index!(mandatory_table::id, mandatory_table::col);
index!(mandatory_table::id, mandatory_table::a_id);
index!(discretionary_table::id, discretionary_table::col);

// Declare singleton foreign keys for discretionary relationships
fpk!(child_table::d1_id -> discretionary_table);
fpk!(child_table::d2_id -> discretionary_table);
fpk!(child_table::d3_id -> discretionary_table);

// FKs from child to intermediates — these macros define the same composite
// foreign keys that are present in the SQL test DDL and allow the builder
// to reason about composite relationships when assembling insert bundles.
fk!((child_table::m1_id, child_table::m1_col) -> (mandatory_table::id, mandatory_table::col));
fk!((child_table::m2_id, child_table::m2_col) -> (mandatory_table::id, mandatory_table::col));
fk!((child_table::m3_id, child_table::m3_col) -> (mandatory_table::id, mandatory_table::col));

fk!((child_table::d1_id, child_table::d1_col) -> (discretionary_table::id, discretionary_table::col));
fk!((child_table::d2_id, child_table::d2_col) -> (discretionary_table::id, discretionary_table::col));
fk!((child_table::d3_id, child_table::d3_col) -> (discretionary_table::id, discretionary_table::col));

/// Map child-side host columns `(id, m1_col)` to the intermediate table's
/// `(a_id, col)`, so that horizontal same-as relationships `(child.id == a_id)`
/// are tracked by the builder when composing nested inserts.
impl diesel_builders::HorizontalKey for child_table::m1_id {
    type HostColumns = (child_table::id, child_table::m1_col);
    type ForeignColumns = (mandatory_table::a_id, mandatory_table::col);
}
/// Horizontal mapping for the m2 mandatory relation; identical in intent to
/// `m1_id` but pointing to the `m2` column set.
impl diesel_builders::HorizontalKey for child_table::m2_id {
    type HostColumns = (child_table::id, child_table::m2_col);
    type ForeignColumns = (mandatory_table::a_id, mandatory_table::col);
}
/// Horizontal mapping for the m3 mandatory relation.
impl diesel_builders::HorizontalKey for child_table::m3_id {
    type HostColumns = (child_table::id, child_table::m3_col);
    type ForeignColumns = (mandatory_table::a_id, mandatory_table::col);
}

/// Horizontal mapping for the d1 discretionary relation. Because the
/// intermediate `col` column is nullable, the mapping respects optional
/// same-as semantics.
impl diesel_builders::HorizontalKey for child_table::d1_id {
    type HostColumns = (child_table::id, child_table::d1_col);
    type ForeignColumns = (discretionary_table::a_id, discretionary_table::col);
}
/// Horizontal mapping for the d2 discretionary relation.
impl diesel_builders::HorizontalKey for child_table::d2_id {
    type HostColumns = (child_table::id, child_table::d2_col);
    type ForeignColumns = (discretionary_table::a_id, discretionary_table::col);
}
/// Horizontal mapping for the d3 discretionary relation.
impl diesel_builders::HorizontalKey for child_table::d3_id {
    type HostColumns = (child_table::id, child_table::d3_col);
    type ForeignColumns = (discretionary_table::a_id, discretionary_table::col);
}

#[test]
/// Insert a child record and validate triangular relationships.
///
/// This test performs a single fluent builder insertion that creates a
/// `Parent`, the required intermediate rows (mandatory & discretionary), and
/// a `Child` row that references them. The test verifies that the child has
/// been correctly inserted and that horizontal tagging and composite FKs
/// work as expected.
fn test_triangular_many() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;
    diesel::sql_query(
        "CREATE TABLE parent_table (
			id INTEGER PRIMARY KEY NOT NULL,
			name TEXT NOT NULL
		)",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE mandatory_table (
			id INTEGER PRIMARY KEY NOT NULL,
			a_id INTEGER NOT NULL REFERENCES parent_table(id),
			col TEXT,
			UNIQUE(id, a_id),
			UNIQUE(id, col)
		)",
    )
    .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE discretionary_table (
			id INTEGER PRIMARY KEY NOT NULL,
			a_id INTEGER NOT NULL REFERENCES parent_table(id),
			col TEXT,
			UNIQUE(id, col)
		)",
    )
    .execute(&mut conn)?;

    // Child table references intermediates; child id is PK and FK to parent
    diesel::sql_query(
        "CREATE TABLE child_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            m1_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			m1_col TEXT,
			m2_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			m2_col TEXT,
			m3_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			m3_col TEXT,
            d1_id INTEGER NOT NULL REFERENCES discretionary_table(id),
			d1_col TEXT,
			d2_id INTEGER NOT NULL REFERENCES discretionary_table(id),
			d2_col TEXT,
			d3_id INTEGER NOT NULL REFERENCES discretionary_table(id),
			d3_col TEXT,
            payload TEXT NOT NULL,
            FOREIGN KEY (m1_id, m1_col) REFERENCES mandatory_table(id, col),
			FOREIGN KEY (m1_id, id) REFERENCES mandatory_table(id, a_id),
            FOREIGN KEY (m2_id, m2_col) REFERENCES mandatory_table(id, col),
			FOREIGN KEY (m2_id, id) REFERENCES mandatory_table(id, a_id),
            FOREIGN KEY (m3_id, m3_col) REFERENCES mandatory_table(id, col),
			FOREIGN KEY (m3_id, id) REFERENCES mandatory_table(id, a_id),
            FOREIGN KEY (d1_id, d1_col) REFERENCES discretionary_table(id, col),
            FOREIGN KEY (d2_id, d2_col) REFERENCES discretionary_table(id, col),
            FOREIGN KEY (d3_id, d3_col) REFERENCES discretionary_table(id, col)
        )",
    )
    .execute(&mut conn)?;

    let other_parent = parent_table::table::builder()
        .name("Other parent")
        .insert(&mut conn)
        .unwrap();

    let discretionary = discretionary_table::table::builder()
        .a_id(other_parent.get_column::<parent_table::id>())
        .insert(&mut conn)
        .unwrap();

    // Now single fluent insert for the child
    let child = child_table::table::builder()
        .m1(mandatory_table::table::builder().col("M1 for Child"))
        .m2(mandatory_table::table::builder().col("M2 for Child"))
        .m3(mandatory_table::table::builder().col("M3 for Child"))
        .d1_model(&discretionary)
        .d2(discretionary_table::table::builder().col("D2 for Child".to_owned()))
        .d3_model(&discretionary)
        .payload("payload")
        .name("Parent of Child")
        .insert(&mut conn)
        .unwrap();

    // Validate insertion
    assert_eq!(child.get_column::<child_table::payload>(), "payload");
    Ok(())
}
