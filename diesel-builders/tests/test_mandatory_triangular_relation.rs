//! Submodule to test a mandatory triangular relation between tables.

mod shared;
mod shared_triangular;
use std::convert::Infallible;

use diesel::{associations::HasTable, prelude::*};
use diesel_builders::{
    DynTypedColumn, IncompleteBuilderError, TableBuilder, TableBuilderBundle, prelude::*,
};
use diesel_builders_derive::TableModel;
use shared_triangular::*;

#[derive(Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(error=ErrorChildWithMandatory, ancestors = parent_table)]
#[diesel(table_name = child_with_satellite_table)]
/// Model for child table with mandatory triangular relation.
pub struct ChildWithMandatory {
    #[infallible]
    #[same_as(satellite_table::parent_id)]
    /// Primary key.
    id: i32,
    #[infallible]
    #[mandatory(shared_triangular::satellite_table)]
    /// Foreign key to table A.
    mandatory_id: i32,
    #[infallible]
    /// Column B value.
    r#type: String,
    #[same_as(satellite_table::field)]
    #[table_model(sql_name = "columns")]
    /// The remote `field` value from table C that B references via
    /// `mandatory_id`. This field is called `columns` to ensure that fields
    /// that have collisions with diesel keywords are handled correctly.
    __columns: Option<String>,
    /// Another remote column from `satellite_table`.
    #[infallible]
    #[same_as(shared_triangular::satellite_table::another_field)]
    another_remote_column: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = simple_child_with_satellite_table)]
/// Model for simple child table with mandatory triangular relation.
pub struct SimpleChildWithMandatory {
    #[same_as(satellite_table::parent_id)]
    /// Primary key.
    id: i32,
    #[mandatory(satellite_table)]
    /// Foreign key to table A.
    mandatory_id: i32,
}

#[derive(Debug, PartialEq, thiserror::Error)]
/// Errors for `NewChildWithMandatory` validation.
pub enum ErrorChildWithMandatory {
    /// `remote_field` cannot be empty.
    #[error("`remote_field` cannot be empty")]
    EmptyRemoteColumnC,
}

impl From<Infallible> for ErrorChildWithMandatory {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl ValidateColumn<child_with_satellite_table::__columns>
    for <child_with_satellite_table::table as TableExt>::NewValues
{
    type Error = ErrorChildWithMandatory;

    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            return Err(ErrorChildWithMandatory::EmptyRemoteColumnC);
        }
        Ok(())
    }
}

#[test]
fn test_mandatory_triangular_relation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    setup_triangular_tables(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE child_with_satellite_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            mandatory_id INTEGER NOT NULL REFERENCES satellite_table(id),
            type TEXT NOT NULL,
            columns TEXT CHECK (columns <> ''),
            another_remote_column TEXT,
			FOREIGN KEY (mandatory_id, id) REFERENCES satellite_table(id, parent_id),
            FOREIGN KEY (mandatory_id, columns) REFERENCES satellite_table(id, field),
            FOREIGN KEY (mandatory_id, another_remote_column) REFERENCES satellite_table(id, another_field)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder().parent_field("Value A").insert(&mut conn).unwrap();

    assert_eq!(parent.parent_field(), "Value A");

    // Insert into table C (references A)
    let mandatory = satellite_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .field("Value C")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(mandatory.field(), "Value C");
    assert_eq!(*mandatory.parent_id(), parent.get_column::<parent_table::id>());

    let mut mandatory_builder =
        satellite_table::table::builder().another_field("Original another remote field".to_owned());
    mandatory_builder.field_ref("Value C");

    // Insert into table B (extends C and references A)
    // The mandatory triangular relation means B's parent_id should automatically
    // match C's parent_id when we only set C's columns
    // Using generated trait methods like try_mandatory_ref for type-safe builders
    let mut child_builder =
        child_with_satellite_table::table::builder().parent_field("Value A for B");

    let saved_child_builder = child_builder.clone();

    assert_eq!(
        child_builder.try_set_mandatory_builder_ref::<child_with_satellite_table::mandatory_id>(
            satellite_table::table::builder().field(String::new())
        ),
        Err(ErrorChildWithMandatory::EmptyRemoteColumnC)
    );

    // Since the operation has failed, the preliminary state of the builder should
    // have remained unchanged.
    assert_eq!(child_builder, saved_child_builder);

    child_builder.try_mandatory_ref(mandatory_builder.clone())?.type_ref("Value B");

    // Using the generated trait method for more ergonomic code
    let child = child_builder
        .try_mandatory(mandatory_builder)?
        // Overriding the another_remote_column value set in the mandatory builder
        .another_remote_column("Another remote field".to_owned())
        .try_columns("After setting mandatory".to_owned())?
        .try_another_remote_column("Another remote field".to_owned())?
        .insert(&mut conn)
        .unwrap();

    let associated_parent: Parent = child.ancestor::<Parent>(&mut conn)?;
    assert_eq!(associated_parent.parent_field(), "Value A for B");

    let associated_mandatory: Satellite = child.mandatory(&mut conn)?;
    assert_eq!(associated_mandatory.field(), "After setting mandatory");
    assert_eq!(associated_mandatory.another_field().as_deref(), Some("Another remote field"));
    assert_eq!(
        *associated_mandatory.parent_id(),
        child.get_column::<child_with_satellite_table::id>()
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        associated_parent.get_column::<parent_table::id>()
    );

    let _ = TableBuilderBundle::<child_with_satellite_table::table>::table();
    let _ = TableBuilder::<child_with_satellite_table::table>::table();

    // Test iter_foreign_keys with composite index (satellite_table::id,
    // satellite_table::field)
    let refs: Vec<_> =
        child.iter_foreign_keys::<(satellite_table::id, satellite_table::field)>().collect();
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(&(child.mandatory_id(), child.__columns.as_ref())));

    // Create a child with mandatory using dynamic column setting
    let dyn_type_column: Box<
        dyn DynTypedColumn<Table = child_with_satellite_table::table, ValueType = String>,
    > = Box::new(child_with_satellite_table::r#type);

    let satellite_builder = satellite_table::table::builder().field("Dynamic Field");

    let child = child_with_satellite_table::table::builder()
        .try_set_dynamic_column(dyn_type_column, "Dynamic Type")?
        .parent_field("Parent for Dynamic Child")
        .try_mandatory(satellite_builder)?
        .insert(&mut conn)?;

    assert_eq!(child.r#type(), "Dynamic Type");
    assert_eq!(child.columns().as_deref(), Some("Dynamic Field"));

    Ok(())
}

#[test]
fn test_mandatory_triangular_relation_simple() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    setup_triangular_tables(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE simple_child_with_satellite_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            mandatory_id INTEGER NOT NULL REFERENCES satellite_table(id),
			FOREIGN KEY (mandatory_id, id) REFERENCES satellite_table(id, parent_id)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder().parent_field("Value A").insert(&mut conn).unwrap();

    assert_eq!(parent.parent_field(), "Value A");

    // Insert into table C (references A)
    let mandatory = satellite_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .field("Value C")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(mandatory.field(), "Value C");
    assert_eq!(*mandatory.parent_id(), parent.get_column::<parent_table::id>());

    let mut mandatory_builder =
        satellite_table::table::builder().another_field("Original another remote field".to_owned());
    mandatory_builder.field_ref("Value C");

    // Insert into table B (extends C and references A)
    // The mandatory triangular relation means B's parent_id should automatically
    // match C's parent_id when we only set C's columns
    // Using generated trait methods like try_mandatory_ref for type-safe builders
    let mut child_builder =
        simple_child_with_satellite_table::table::builder().parent_field("Value A for B");

    let saved_child_builder = child_builder.clone();

    // Since the operation has failed, the preliminary state of the builder should
    // have remained unchanged.
    assert_eq!(child_builder, saved_child_builder);

    child_builder.try_mandatory_ref(mandatory_builder.clone())?;

    // Using the generated trait method for more ergonomic code
    let child = child_builder.try_mandatory(mandatory_builder)?.insert(&mut conn).unwrap();

    let associated_parent: Parent = child.ancestor::<Parent>(&mut conn)?;
    assert_eq!(associated_parent.parent_field(), "Value A for B");

    let associated_mandatory: Satellite = child.mandatory(&mut conn)?;
    assert_eq!(associated_mandatory.field(), "Value C");
    assert_eq!(
        associated_mandatory.another_field().as_deref(),
        Some("Original another remote field")
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        child.get_column::<simple_child_with_satellite_table::id>()
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        associated_parent.get_column::<parent_table::id>()
    );

    // Test iter_foreign_key_columns with composite index (satellite_table::id,
    // satellite_table::parent_id)
    let refs: Vec<_> = child
        .iter_foreign_key_columns::<(satellite_table::id, satellite_table::parent_id)>()
        .collect();
    assert_eq!(refs.len(), 1);
    // iter_foreign_key_columns yields boxed host table columns
    // We can verify the count but can't directly inspect the boxed trait objects

    Ok(())
}

#[test]
fn test_mandatory_triangular_relation_missing_builder_error() {
    use std::convert::TryFrom;

    use diesel_builders::{CompletedTableBuilderBundle, TableBuilderBundle};

    // Create a TableBuilderBundle without setting the mandatory associated builder
    let b_bundle = TableBuilderBundle::<child_with_satellite_table::table>::default();

    // Try to convert to CompletedTableBuilderBundle - this should fail because
    // the mandatory associated builder for mandatory_id has not been set
    let result = CompletedTableBuilderBundle::try_from(b_bundle);

    // Verify that the conversion fails with the expected error message
    let err = result.unwrap_err();
    assert_eq!(err, IncompleteBuilderError::MissingMandatoryTriangularField("mandatory_id"));
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder with mandatory triangular relation
    let builder = child_with_satellite_table::table::builder()
        .r#type("Serialized B")
        .try_columns("Serialized C".to_string())?;

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<child_with_satellite_table::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(
        deserialized.may_get_column_ref::<child_with_satellite_table::r#type>().map(String::as_str),
        Some("Serialized B")
    );
    assert_eq!(
        deserialized.may_get_column_ref::<child_with_satellite_table::__columns>(),
        Some(&Some("Serialized C".to_string()))
    );

    Ok(())
}
