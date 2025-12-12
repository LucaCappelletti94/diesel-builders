//! Test case for a table with both mandatory and discretionary triangular relations.
//!
//! This test sets up four tables: Parent (root), Mandatory (references Parent), Discretionary (references Parent),
//! and `ChildWithMixed` which:
//! - Has a mandatory triangular relation with Mandatory (via `mandatory_id`)
//! - Has a discretionary triangular relation with Discretionary (via `discretionary_id`)

mod shared;
mod shared_triangular;
use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::TableModel;
use shared_triangular::*;

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = child_with_mixed_table)]
#[table_model(ancestors = parent_table)]
/// Model for table B.
pub struct ChildWithMixed {
    /// Primary key.
    id: i32,
    #[mandatory]
    /// Foreign key to table C.
    mandatory_id: i32,
    #[discretionary]
    /// Foreign key to table D.
    discretionary_id: i32,
    /// Column B value.
    child_field: String,
    /// Remote column C value.
    remote_mandatory_field: Option<String>,
    /// Remote column D value.
    remote_discretionary_field: Option<String>,
}

// Define foreign key relationships
fk!((child_with_mixed_table::mandatory_id, child_with_mixed_table::remote_mandatory_field) -> (mandatory_table::id, mandatory_table::mandatory_field));
fk!((child_with_mixed_table::mandatory_id, child_with_mixed_table::id) -> (mandatory_table::id, mandatory_table::parent_id));
fk!((child_with_mixed_table::discretionary_id, child_with_mixed_table::remote_discretionary_field) -> (discretionary_table::id, discretionary_table::discretionary_field));

// Define horizontal same-as relationships
impl diesel_builders::HorizontalKey for child_with_mixed_table::mandatory_id {
    type HostColumns = (
        child_with_mixed_table::id,
        child_with_mixed_table::remote_mandatory_field,
    );
    type ForeignColumns = (mandatory_table::parent_id, mandatory_table::mandatory_field);
}

impl diesel_builders::HorizontalKey for child_with_mixed_table::discretionary_id {
    type HostColumns = (
        child_with_mixed_table::id,
        child_with_mixed_table::remote_discretionary_field,
    );
    type ForeignColumns = (
        discretionary_table::parent_id,
        discretionary_table::discretionary_field,
    );
}

fn create_tables(conn: &mut SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    setup_triangular_tables(conn)?;

    diesel::sql_query(
        "CREATE TABLE child_with_mixed_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            mandatory_id INTEGER NOT NULL REFERENCES mandatory_table(id),
            discretionary_id INTEGER NOT NULL REFERENCES discretionary_table(id),
            child_field TEXT NOT NULL,
            remote_mandatory_field TEXT,
            remote_discretionary_field TEXT,
            FOREIGN KEY (mandatory_id, id) REFERENCES mandatory_table(id, parent_id),
            FOREIGN KEY (mandatory_id, remote_mandatory_field) REFERENCES mandatory_table(id, mandatory_field),
            FOREIGN KEY (discretionary_id, remote_discretionary_field) REFERENCES discretionary_table(id, discretionary_field)
        )",
    )
    .execute(conn)?;

    Ok(())
}

#[test]
fn test_mixed_triangular_relations() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    // Insert B with both mandatory C and discretionary D
    // Using generated trait methods for ergonomic builder setup
    let b = child_with_mixed_table::table::builder()
        .parent_field("Value A for B")
        .child_field("Value B")
        .mandatory(mandatory_table::table::builder().mandatory_field(Some("Value C".to_owned())))
        .discretionary(
            discretionary_table::table::builder().discretionary_field(Some("Value D".to_owned())),
        )
        .insert(&mut conn)?;

    assert_eq!(
        b.get_column::<child_with_mixed_table::child_field>(),
        "Value B"
    );
    assert_eq!(
        b.get_column::<child_with_mixed_table::remote_mandatory_field>()
            .as_deref(),
        Some("Value C")
    );
    assert_eq!(
        b.get_column::<child_with_mixed_table::remote_discretionary_field>()
            .as_deref(),
        Some("Value D")
    );

    // Verify associated C
    let c: Mandatory = b.mandatory(&mut conn)?;
    assert_eq!(
        c.get_column::<mandatory_table::parent_id>(),
        b.get_column::<child_with_mixed_table::id>()
    );
    assert_eq!(
        c.get_column::<mandatory_table::mandatory_field>()
            .as_deref(),
        Some("Value C")
    );

    // Verify associated D
    let d: Discretionary = b.discretionary(&mut conn)?;
    assert_eq!(
        d.get_column::<discretionary_table::parent_id>(),
        b.get_column::<child_with_mixed_table::id>()
    );
    assert_eq!(
        d.get_column::<discretionary_table::discretionary_field>()
            .as_deref(),
        Some("Value D")
    );

    Ok(())
}

#[test]
fn test_get_foreign_ext_direct() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    let b = child_with_mixed_table::table::builder()
        .parent_field("Value A for B")
        .child_field("Value B")
        .mandatory(mandatory_table::table::builder().mandatory_field(Some("Value C".to_owned())))
        .discretionary(
            discretionary_table::table::builder().discretionary_field(Some("Value D".to_owned())),
        )
        .insert(&mut conn)?;

    // Use GetForeignExt directly for primary-key based foreign key
    let c_pk: Mandatory = b
        .get_foreign::<(child_with_mixed_table::mandatory_id,), (mandatory_table::id,)>(
            &mut conn,
        )?;
    assert_eq!(
        c_pk.get_column::<mandatory_table::id>(),
        b.get_column::<child_with_mixed_table::mandatory_id>()
    );
    assert_eq!(
        c_pk.get_column::<mandatory_table::mandatory_field>()
            .as_deref(),
        Some("Value C")
    );

    // Use GetForeignExt directly for composite foreign key mapping (non-nullable types)
    let c_horizontal: Mandatory =
        b.get_foreign::<(
            child_with_mixed_table::mandatory_id,
            child_with_mixed_table::id,
        ), (mandatory_table::id, mandatory_table::parent_id)>(&mut conn)?;
    assert_eq!(
        c_horizontal.get_column::<mandatory_table::id>(),
        b.get_column::<child_with_mixed_table::mandatory_id>()
    );
    assert_eq!(
        c_horizontal.get_column::<mandatory_table::parent_id>(),
        b.get_column::<child_with_mixed_table::id>()
    );
    assert_eq!(
        c_horizontal
            .get_column::<mandatory_table::mandatory_field>()
            .as_deref(),
        Some("Value C")
    );

    Ok(())
}

#[test]
fn test_mixed_triangular_missing_mandatory_fails() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    let parent_table = parent_table::table::builder()
        .parent_field("Value A")
        .insert(&mut conn)?;

    let discretionary_table = discretionary_table::table::builder()
        .parent_id(parent_table.get_column::<parent_table::id>())
        .discretionary_field(Some("Value D".to_owned()))
        .insert(&mut conn)?;

    // Try to create without mandatory C builder
    // Note: d_id_model references an existing model instead of creating a new one
    let result = child_with_mixed_table::table::builder()
        .parent_field("Value A")
        .child_field("Value B")
        .discretionary(
            discretionary_table::table::builder().discretionary_field(Some("Value D".to_owned())),
        )
        .discretionary_model(&discretionary_table)
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
    let builder = child_with_mixed_table::table::builder()
        .child_field("Serialized B")
        .try_remote_mandatory_field(Some("Serialized C".to_string()))?
        .try_remote_discretionary_field(Some("Serialized D".to_string()))?;

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<child_with_mixed_table::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(
        deserialized
            .may_get_column_ref::<child_with_mixed_table::child_field>()
            .map(String::as_str),
        Some("Serialized B")
    );
    assert_eq!(
        deserialized.may_get_column_ref::<child_with_mixed_table::remote_mandatory_field>(),
        Some(&Some("Serialized C".to_string()))
    );
    assert_eq!(
        deserialized.may_get_column_ref::<child_with_mixed_table::remote_discretionary_field>(),
        Some(&Some("Serialized D".to_string()))
    );

    Ok(())
}
