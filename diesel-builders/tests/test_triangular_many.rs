//! Test for a child table with multiple mandatory and discretionary triangular relations.

mod shared;
mod shared_triangular;
use diesel_builders::prelude::*;
use diesel_builders_macros::TableModel;
use shared_triangular::*;

/// The `Child` table ties together multiple intermediate rows using
/// composite foreign keys. The `payload` field verifies the builder's
/// insertion logic while several `m*` and `d*` columns test mandatory and
/// discretionary triangular relationships.
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(ancestors = parent_table)]
pub struct Child {
    /// The primary key of the child, also a foreign key to `parent_table`.
    id: i32,
    #[mandatory(mandatory_table)]
    /// Mandatory relation 1 ID.
    m1_id: i32,
    /// Secondary column used by composite FKs for `Mandatory` intermediates.
    m1_mandatory_field: Option<String>,
    #[mandatory(mandatory_table)]
    /// Mandatory relation 2 ID.
    m2_id: i32,
    /// Mandatory relation 2 column.
    m2_mandatory_field: Option<String>,
    #[mandatory(mandatory_table)]
    /// Mandatory relation 3 ID.
    m3_id: i32,
    /// Mandatory relation 3 column.
    m3_mandatory_field: Option<String>,
    #[discretionary(discretionary_table)]
    /// Discretionary relation 1 ID.
    d1_id: i32,
    /// Optional secondary column used by composite FKs for discretionary
    /// intermediates; `None` denotes an unset optional value.
    d1_discretionary_field: Option<String>,
    #[discretionary(discretionary_table)]
    /// Discretionary relation 2 ID.
    d2_id: i32,
    /// Discretionary relation 2 column.
    d2_discretionary_field: Option<String>,
    #[discretionary(discretionary_table)]
    /// Discretionary relation 3 ID.
    d3_id: i32,
    /// Discretionary relation 3 column.
    d3_discretionary_field: Option<String>,
    /// The payload establishes a simple observable column for test
    /// assertions after insert.
    payload: String,
}

// FKs from child to intermediates — these macros define the same composite
// foreign keys that are present in the SQL test DDL and allow the builder
// to reason about composite relationships when assembling insert bundles.
fk!((child_table::m1_id, child_table::m1_mandatory_field) -> (mandatory_table::id, mandatory_table::mandatory_field));
fk!((child_table::m1_id, child_table::id) -> (mandatory_table::id, mandatory_table::parent_id));
fk!((child_table::m2_id, child_table::m2_mandatory_field) -> (mandatory_table::id, mandatory_table::mandatory_field));
fk!((child_table::m2_id, child_table::id) -> (mandatory_table::id, mandatory_table::parent_id));
fk!((child_table::m3_id, child_table::m3_mandatory_field) -> (mandatory_table::id, mandatory_table::mandatory_field));
fk!((child_table::m3_id, child_table::id) -> (mandatory_table::id, mandatory_table::parent_id));

fk!((child_table::d1_id, child_table::d1_discretionary_field) -> (discretionary_table::id, discretionary_table::discretionary_field));
fk!((child_table::d2_id, child_table::d2_discretionary_field) -> (discretionary_table::id, discretionary_table::discretionary_field));
fk!((child_table::d3_id, child_table::d3_discretionary_field) -> (discretionary_table::id, discretionary_table::discretionary_field));

/// Map child-side host columns `(id, m1_col)` to the intermediate table's
/// `(a_id, col)`, so that horizontal same-as relationships `(child.id == a_id)`
/// are tracked by the builder when composing nested inserts.
impl diesel_builders::HorizontalKey for child_table::m1_id {
    type HostColumns = (child_table::id, child_table::m1_mandatory_field);
    type ForeignColumns = (mandatory_table::parent_id, mandatory_table::mandatory_field);
}
/// Horizontal mapping for the m2 mandatory relation; identical in intent to
/// `m1_id` but pointing to the `m2` column set.
impl diesel_builders::HorizontalKey for child_table::m2_id {
    type HostColumns = (child_table::id, child_table::m2_mandatory_field);
    type ForeignColumns = (mandatory_table::parent_id, mandatory_table::mandatory_field);
}
/// Horizontal mapping for the m3 mandatory relation.
impl diesel_builders::HorizontalKey for child_table::m3_id {
    type HostColumns = (child_table::id, child_table::m3_mandatory_field);
    type ForeignColumns = (mandatory_table::parent_id, mandatory_table::mandatory_field);
}

/// Horizontal mapping for the d1 discretionary relation. Because the
/// intermediate `col` column is nullable, the mapping respects optional
/// same-as semantics.
impl diesel_builders::HorizontalKey for child_table::d1_id {
    type HostColumns = (child_table::id, child_table::d1_discretionary_field);
    type ForeignColumns = (
        discretionary_table::parent_id,
        discretionary_table::discretionary_field,
    );
}
/// Horizontal mapping for the d2 discretionary relation.
impl diesel_builders::HorizontalKey for child_table::d2_id {
    type HostColumns = (child_table::id, child_table::d2_discretionary_field);
    type ForeignColumns = (
        discretionary_table::parent_id,
        discretionary_table::discretionary_field,
    );
}
/// Horizontal mapping for the d3 discretionary relation.
impl diesel_builders::HorizontalKey for child_table::d3_id {
    type HostColumns = (child_table::id, child_table::d3_discretionary_field);
    type ForeignColumns = (
        discretionary_table::parent_id,
        discretionary_table::discretionary_field,
    );
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
    let mut conn = shared::establish_connection()?;
    setup_triangular_tables(&mut conn)?;

    // Child table references intermediates; child id is PK and FK to parent
    diesel::sql_query(
        "CREATE TABLE child_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            m1_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			m1_mandatory_field TEXT,
			m2_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			m2_mandatory_field TEXT,
			m3_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			m3_mandatory_field TEXT,
            d1_id INTEGER NOT NULL REFERENCES discretionary_table(id),
			d1_discretionary_field TEXT,
			d2_id INTEGER NOT NULL REFERENCES discretionary_table(id),
			d2_discretionary_field TEXT,
			d3_id INTEGER NOT NULL REFERENCES discretionary_table(id),
			d3_discretionary_field TEXT,
            payload TEXT NOT NULL,
            FOREIGN KEY (m1_id, m1_mandatory_field) REFERENCES mandatory_table(id, mandatory_field),
			FOREIGN KEY (m1_id, id) REFERENCES mandatory_table(id, parent_id),
            FOREIGN KEY (m2_id, m2_mandatory_field) REFERENCES mandatory_table(id, mandatory_field),
			FOREIGN KEY (m2_id, id) REFERENCES mandatory_table(id, parent_id),
            FOREIGN KEY (m3_id, m3_mandatory_field) REFERENCES mandatory_table(id, mandatory_field),
			FOREIGN KEY (m3_id, id) REFERENCES mandatory_table(id, parent_id),
            FOREIGN KEY (d1_id, d1_discretionary_field) REFERENCES discretionary_table(id, discretionary_field),
            FOREIGN KEY (d2_id, d2_discretionary_field) REFERENCES discretionary_table(id, discretionary_field),
            FOREIGN KEY (d3_id, d3_discretionary_field) REFERENCES discretionary_table(id, discretionary_field)
        )",
    )
    .execute(&mut conn)?;

    let other_parent = parent_table::table::builder()
        .parent_field("Other parent")
        .insert(&mut conn)
        .unwrap();

    let discretionary = discretionary_table::table::builder()
        .parent_id(other_parent.get_column::<parent_table::id>())
        .insert(&mut conn)
        .unwrap();

    // Now single fluent insert for the child
    let child = child_table::table::builder()
        .m1(mandatory_table::table::builder().mandatory_field(Some("M1 for Child".to_owned())))
        .m2(mandatory_table::table::builder().mandatory_field(Some("M2 for Child".to_owned())))
        .m3(mandatory_table::table::builder().mandatory_field(Some("M3 for Child".to_owned())))
        .d1_model(&discretionary)
        .d2(discretionary_table::table::builder()
            .discretionary_field(Some("D2 for Child".to_owned())))
        .d3_model(&discretionary)
        .payload("payload")
        .parent_field("Parent of Child")
        .insert(&mut conn)
        .unwrap();

    // Validate insertion
    assert_eq!(child.get_column::<child_table::payload>(), "payload");
    Ok(())
}
