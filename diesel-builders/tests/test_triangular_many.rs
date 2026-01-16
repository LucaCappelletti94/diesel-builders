//! Test for a child table with multiple mandatory and discretionary triangular
//! relations.

mod shared;
mod shared_triangular;
use diesel_builders::prelude::*;
use diesel_builders_derive::TableModel;
use shared_triangular::*;

/// The `Child` table ties together multiple intermediate rows using
/// composite foreign keys. The `payload` field verifies the builder's
/// insertion logic while several `m*` and `d*` columns test mandatory and
/// discretionary triangular relationships.
#[derive(Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(ancestors = parent_table)]
pub struct Child {
    #[same_as(satellite_table::parent_id, (m1_id, m2_id, m3_id))]
    #[same_as(satellite_table::parent_id, (d1_id, d2_id, d3_id))]
    /// The primary key of the child, also a foreign key to `parent_table`.
    id: i32,
    #[mandatory(satellite_table)]
    /// Mandatory relation 1 ID.
    m1_id: i32,
    #[same_as(satellite_table::field, m1_id)]
    /// Secondary column used by composite FKs for `Mandatory` intermediates.
    m1_field: Option<String>,
    #[mandatory(satellite_table)]
    /// Mandatory relation 2 ID.
    m2_id: i32,
    #[same_as(satellite_table::field, m2_id)]
    /// Mandatory relation 2 column.
    m2_field: Option<String>,
    #[mandatory(satellite_table)]
    /// Mandatory relation 3 ID.
    m3_id: i32,
    #[same_as(satellite_table::field, m3_id)]
    /// Mandatory relation 3 column.
    m3_field: Option<String>,
    #[discretionary(satellite_table)]
    /// Discretionary relation 1 ID.
    d1_id: i32,
    #[same_as(satellite_table::field, d1_id)]
    /// Optional secondary column used by composite FKs for discretionary
    /// intermediates; `None` denotes an unset optional value.
    d1_field: Option<String>,
    #[discretionary(satellite_table)]
    /// Discretionary relation 2 ID.
    d2_id: i32,
    #[same_as(satellite_table::field, d2_id)]
    /// Discretionary relation 2 column.
    d2_field: Option<String>,
    #[discretionary(satellite_table)]
    /// Discretionary relation 3 ID.
    d3_id: i32,
    #[same_as(satellite_table::field, d3_id)]
    /// Discretionary relation 3 column.
    d3_field: Option<String>,
    /// The payload establishes a simple observable column for test
    /// assertions after insert.
    payload: String,
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
            m1_id INTEGER NOT NULL REFERENCES satellite_table(id),
			m1_field TEXT,
			m2_id INTEGER NOT NULL REFERENCES satellite_table(id),
			m2_field TEXT,
			m3_id INTEGER NOT NULL REFERENCES satellite_table(id),
			m3_field TEXT,
            d1_id INTEGER NOT NULL REFERENCES satellite_table(id),
			d1_field TEXT,
			d2_id INTEGER NOT NULL REFERENCES satellite_table(id),
			d2_field TEXT,
			d3_id INTEGER NOT NULL REFERENCES satellite_table(id),
			d3_field TEXT,
            payload TEXT NOT NULL,
            FOREIGN KEY (m1_id, m1_field) REFERENCES satellite_table(id, field),
			FOREIGN KEY (m1_id, id) REFERENCES satellite_table(id, parent_id),
            FOREIGN KEY (m2_id, m2_field) REFERENCES satellite_table(id, field),
			FOREIGN KEY (m2_id, id) REFERENCES satellite_table(id, parent_id),
            FOREIGN KEY (m3_id, m3_field) REFERENCES satellite_table(id, field),
			FOREIGN KEY (m3_id, id) REFERENCES satellite_table(id, parent_id),
            FOREIGN KEY (d1_id, d1_field) REFERENCES satellite_table(id, field),
            FOREIGN KEY (d2_id, d2_field) REFERENCES satellite_table(id, field),
            FOREIGN KEY (d3_id, d3_field) REFERENCES satellite_table(id, field)
        )",
    )
    .execute(&mut conn)?;

    let other_parent =
        parent_table::table::builder().parent_field("Other parent").insert(&mut conn)?;

    let discretionary = satellite_table::table::builder()
        .parent_id(other_parent.get_column::<parent_table::id>())
        .field("Discretionary for Child")
        .insert(&mut conn)?;

    // Now single fluent insert for the child
    let child = child_table::table::builder()
        .m1(satellite_table::table::builder().field("M1 for Child"))
        .m2(satellite_table::table::builder().field("M2 for Child"))
        .m3(satellite_table::table::builder().field("M3 for Child"))
        .d1_model(&discretionary)
        .d2(satellite_table::table::builder().field("D2 for Child"))
        .d3_model(&discretionary)
        .payload("payload")
        .parent_field("Parent of Child")
        .insert(&mut conn)?;

    // Validate insertion
    assert_eq!(child.payload(), "payload");
    let d1 = child.d1(&mut conn)?;
    assert_eq!(d1, discretionary);
    let d2 = child.d2(&mut conn)?;
    assert_eq!(d2.field(), "D2 for Child");
    let d3 = child.d3(&mut conn)?;
    assert_eq!(d3, discretionary);

    // Test iter_foreign_keys - Child has 6 foreign keys to satellite_table (m1, m2,
    // m3, d1, d2, d3) Using composite index (satellite_table::id,
    // satellite_table::field)
    let _m1 = child.m1(&mut conn)?;
    let _m2 = child.m2(&mut conn)?;
    let _m3 = child.m3(&mut conn)?;
    let refs: Vec<_> =
        child.iter_foreign_keys::<(satellite_table::id, satellite_table::field)>().collect();
    assert_eq!(refs.len(), 6);
    assert!(refs.contains(&(child.m1_id(), child.m1_field.as_ref())));
    assert!(refs.contains(&(child.m2_id(), child.m2_field.as_ref())));
    assert!(refs.contains(&(child.m3_id(), child.m3_field.as_ref())));
    assert!(refs.contains(&(child.d1_id(), child.d1_field.as_ref())));
    assert!(refs.contains(&(child.d2_id(), child.d2_field.as_ref())));
    assert!(refs.contains(&(child.d3_id(), child.d3_field.as_ref())));

    Ok(())
}
