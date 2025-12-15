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
    #[same_as(mandatory_table::parent_id, (m1_id, m2_id, m3_id))]
    #[same_as(discretionary_table::parent_id, (d1_id, d2_id, d3_id))]
    /// The primary key of the child, also a foreign key to `parent_table`.
    id: i32,
    #[mandatory(mandatory_table)]
    /// Mandatory relation 1 ID.
    m1_id: i32,
    #[same_as(mandatory_table::mandatory_field, m1_id)]
    /// Secondary column used by composite FKs for `Mandatory` intermediates.
    m1_mandatory_field: Option<String>,
    #[mandatory(mandatory_table)]
    /// Mandatory relation 2 ID.
    m2_id: i32,
    #[same_as(mandatory_table::mandatory_field, m2_id)]
    /// Mandatory relation 2 column.
    m2_mandatory_field: Option<String>,
    #[mandatory(mandatory_table)]
    /// Mandatory relation 3 ID.
    m3_id: i32,
    #[same_as(mandatory_table::mandatory_field, m3_id)]
    /// Mandatory relation 3 column.
    m3_mandatory_field: Option<String>,
    #[discretionary(discretionary_table)]
    /// Discretionary relation 1 ID.
    d1_id: i32,
    #[same_as(discretionary_table::discretionary_field, d1_id)]
    /// Optional secondary column used by composite FKs for discretionary
    /// intermediates; `None` denotes an unset optional value.
    d1_discretionary_field: Option<String>,
    #[discretionary(discretionary_table)]
    /// Discretionary relation 2 ID.
    d2_id: i32,
    #[same_as(discretionary_table::discretionary_field, d2_id)]
    /// Discretionary relation 2 column.
    d2_discretionary_field: Option<String>,
    #[discretionary(discretionary_table)]
    /// Discretionary relation 3 ID.
    d3_id: i32,
    #[same_as(discretionary_table::discretionary_field, d3_id)]
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
        .insert(&mut conn)?;

    let discretionary = discretionary_table::table::builder()
        .parent_id(other_parent.get_column::<parent_table::id>())
        .discretionary_field("Discretionary for Child")
        .insert(&mut conn)?;

    // Now single fluent insert for the child
    let child = child_table::table::builder()
        .m1(mandatory_table::table::builder().mandatory_field("M1 for Child"))
        .m2(mandatory_table::table::builder().mandatory_field("M2 for Child"))
        .m3(mandatory_table::table::builder().mandatory_field("M3 for Child"))
        .d1_model(&discretionary)
        .d2(discretionary_table::table::builder().discretionary_field("D2 for Child"))
        .d3_model(&discretionary)
        .payload("payload")
        .parent_field("Parent of Child")
        .insert(&mut conn)?;

    // Validate insertion
    assert_eq!(child.get_column::<child_table::payload>(), "payload");
    let d1 = child.d1(&mut conn)?;
    assert_eq!(d1, discretionary);
    let d2 = child.d2(&mut conn)?;
    assert_eq!(
        d2.get_column::<discretionary_table::discretionary_field>(),
        "D2 for Child"
    );
    let d3 = child.d3(&mut conn)?;
    assert_eq!(d3, discretionary);
    Ok(())
}
