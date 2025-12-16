//! Submodule to test a mandatory triangular relation between tables.
//!
//! This test sets up three tables: Parent, Child, and Mandatory. Child extends Mandatory, and Mandatory contains
//! a column that references Parent, and Child has a column that references Mandatory, forming a
//! triangular relationship. The test verifies that inserts and queries work
//! correctly through this relationship.
//!
//! Specifically, the relationship is mandatory, that is the foreign key from
//! Mandatory to Parent is referenced in Child using a same-as relationship, which means that
//! the Mandatory record associated with a Child record must reference the same Parent record as
//! Child does. Furthermore, another column in Child is linked via the same-as
//! relationship to a column in Mandatory, value which needs to be set when setting the
//! builder for the Mandatory record in the Child builder.

mod shared;
mod shared_triangular;
use shared_triangular::*;

use std::convert::Infallible;

use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel_builders::{IncompleteBuilderError, TableBuilder, TableBuilderBundle, prelude::*};
use diesel_builders_macros::TableModel;

#[derive(Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(error=ErrorChildWithMandatory, ancestors = parent_table)]
#[diesel(table_name = child_with_mandatory_table)]
/// Model for child table with mandatory triangular relation.
pub struct ChildWithMandatory {
    #[infallible]
    #[same_as(mandatory_table::parent_id)]
    /// Primary key.
    id: i32,
    #[infallible]
    #[mandatory(mandatory_table)]
    /// Foreign key to table A.
    mandatory_id: i32,
    #[infallible]
    /// Column B value.
    child_field: String,
    #[same_as(mandatory_table::mandatory_field)]
    /// The remote `mandatory_field` value from table C that B references via `mandatory_id`.
    remote_mandatory_field: Option<String>,
    /// Another remote column from `mandatory_table`.
    #[infallible]
    #[same_as(mandatory_table::another_field)]
    another_remote_column: Option<String>,
}

fk!((child_with_mandatory_table::mandatory_id, child_with_mandatory_table::id) -> (mandatory_table::id, mandatory_table::parent_id));
fk!((child_with_mandatory_table::mandatory_id, child_with_mandatory_table::remote_mandatory_field) -> (mandatory_table::id, mandatory_table::mandatory_field));
fk!((child_with_mandatory_table::mandatory_id, child_with_mandatory_table::another_remote_column) -> (mandatory_table::id, mandatory_table::another_field));

#[derive(Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = simple_child_with_mandatory_table)]
/// Model for simple child table with mandatory triangular relation.
pub struct SimpleChildWithMandatory {
    #[infallible]
    #[same_as(mandatory_table::parent_id)]
    /// Primary key.
    id: i32,
    #[infallible]
    #[mandatory(mandatory_table)]
    /// Foreign key to table A.
    mandatory_id: i32,
}

fk!((simple_child_with_mandatory_table::mandatory_id, simple_child_with_mandatory_table::id) -> (mandatory_table::id, mandatory_table::parent_id));

#[derive(Debug, PartialEq, thiserror::Error)]
/// Errors for `NewChildWithMandatory` validation.
pub enum ErrorChildWithMandatory {
    /// `remote_mandatory_field` cannot be empty.
    #[error("`remote_mandatory_field` cannot be empty")]
    EmptyRemoteColumnC,
}

impl From<Infallible> for ErrorChildWithMandatory {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl ValidateColumn<child_with_mandatory_table::remote_mandatory_field>
    for <child_with_mandatory_table::table as TableExt>::NewValues
{
    type Error = ErrorChildWithMandatory;
    type Borrowed = str;

    fn validate_column(value: &Self::Borrowed) -> Result<(), Self::Error> {
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
        "CREATE TABLE child_with_mandatory_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            mandatory_id INTEGER NOT NULL REFERENCES mandatory_table(id),
            child_field TEXT NOT NULL,
            remote_mandatory_field TEXT CHECK (remote_mandatory_field <> ''),
            another_remote_column TEXT,
			FOREIGN KEY (mandatory_id, id) REFERENCES mandatory_table(id, parent_id),
            FOREIGN KEY (mandatory_id, remote_mandatory_field) REFERENCES mandatory_table(id, mandatory_field),
            FOREIGN KEY (mandatory_id, another_remote_column) REFERENCES mandatory_table(id, another_field)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder()
        .parent_field("Value A")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(parent.parent_field(), "Value A");

    // Insert into table C (references A)
    let mandatory = mandatory_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .mandatory_field("Value C")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(mandatory.mandatory_field(), "Value C");
    assert_eq!(
        *mandatory.parent_id(),
        parent.get_column::<parent_table::id>()
    );

    let mut mandatory_builder =
        mandatory_table::table::builder().another_field("Original another remote field".to_owned());
    mandatory_builder.mandatory_field_ref("Value C");

    // Insert into table B (extends C and references A)
    // The mandatory triangular relation means B's parent_id should automatically
    // match C's parent_id when we only set C's columns
    // Using generated trait methods like try_mandatory_ref for type-safe builders
    let mut child_builder =
        child_with_mandatory_table::table::builder().parent_field("Value A for B");

    let saved_child_builder = child_builder.clone();

    assert_eq!(
        child_builder.try_set_mandatory_builder_ref::<child_with_mandatory_table::mandatory_id>(
            mandatory_table::table::builder().mandatory_field(String::new())
        ),
        Err(ErrorChildWithMandatory::EmptyRemoteColumnC)
    );

    // Since the operation has failed, the preliminary state of the builder should
    // have remained unchanged.
    assert_eq!(child_builder, saved_child_builder);

    child_builder
        .try_mandatory_ref(mandatory_builder.clone())?
        .child_field_ref("Value B");

    // Using the generated trait method for more ergonomic code
    let child = child_builder
        .try_mandatory(mandatory_builder)?
        // Overriding the another_remote_column value set in the mandatory builder
        .another_remote_column("Another remote field".to_owned())
        .try_remote_mandatory_field("After setting mandatory".to_owned())?
        .try_another_remote_column("Another remote field".to_owned())?
        .insert(&mut conn)
        .unwrap();

    let associated_parent: Parent = child.ancestor::<Parent>(&mut conn)?;
    assert_eq!(associated_parent.parent_field(), "Value A for B");

    let associated_mandatory: Mandatory = child.mandatory(&mut conn)?;
    assert_eq!(
        associated_mandatory.mandatory_field(),
        "After setting mandatory"
    );
    assert_eq!(
        associated_mandatory.another_field().as_deref(),
        Some("Another remote field")
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        child.get_column::<child_with_mandatory_table::id>()
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        associated_parent.get_column::<parent_table::id>()
    );

    let _ = TableBuilderBundle::<child_with_mandatory_table::table>::table();
    let _ = TableBuilder::<child_with_mandatory_table::table>::table();

    Ok(())
}

#[test]
fn test_mandatory_triangular_relation_simple() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    setup_triangular_tables(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE simple_child_with_mandatory_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            mandatory_id INTEGER NOT NULL REFERENCES mandatory_table(id),
			FOREIGN KEY (mandatory_id, id) REFERENCES mandatory_table(id, parent_id)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder()
        .parent_field("Value A")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(parent.parent_field(), "Value A");

    // Insert into table C (references A)
    let mandatory = mandatory_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .mandatory_field("Value C")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(mandatory.mandatory_field(), "Value C");
    assert_eq!(
        *mandatory.parent_id(),
        parent.get_column::<parent_table::id>()
    );

    let mut mandatory_builder =
        mandatory_table::table::builder().another_field("Original another remote field".to_owned());
    mandatory_builder.mandatory_field_ref("Value C");

    // Insert into table B (extends C and references A)
    // The mandatory triangular relation means B's parent_id should automatically
    // match C's parent_id when we only set C's columns
    // Using generated trait methods like try_mandatory_ref for type-safe builders
    let mut child_builder =
        simple_child_with_mandatory_table::table::builder().parent_field("Value A for B");

    let saved_child_builder = child_builder.clone();

    // Since the operation has failed, the preliminary state of the builder should
    // have remained unchanged.
    assert_eq!(child_builder, saved_child_builder);

    child_builder.try_mandatory_ref(mandatory_builder.clone())?;

    // Using the generated trait method for more ergonomic code
    let child = child_builder
        .try_mandatory(mandatory_builder)?
        .insert(&mut conn)
        .unwrap();

    let associated_parent: Parent = child.ancestor::<Parent>(&mut conn)?;
    assert_eq!(associated_parent.parent_field(), "Value A for B");

    let associated_mandatory: Mandatory = child.mandatory(&mut conn)?;
    assert_eq!(associated_mandatory.mandatory_field(), "Value C");
    assert_eq!(
        associated_mandatory.another_field().as_deref(),
        Some("Original another remote field")
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        child.get_column::<simple_child_with_mandatory_table::id>()
    );
    assert_eq!(
        *associated_mandatory.parent_id(),
        associated_parent.get_column::<parent_table::id>()
    );

    Ok(())
}

#[test]
fn test_mandatory_triangular_relation_missing_builder_error() {
    use diesel_builders::{CompletedTableBuilderBundle, TableBuilderBundle};
    use std::convert::TryFrom;

    // Create a TableBuilderBundle without setting the mandatory associated builder
    let b_bundle = TableBuilderBundle::<child_with_mandatory_table::table>::default();

    // Try to convert to CompletedTableBuilderBundle - this should fail because
    // the mandatory associated builder for mandatory_id has not been set
    let result = CompletedTableBuilderBundle::try_from(b_bundle);

    // Verify that the conversion fails with the expected error message
    let err = result.unwrap_err();
    assert_eq!(
        err,
        IncompleteBuilderError::MissingMandatoryTriangularField("mandatory_id")
    );
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder with mandatory triangular relation
    let builder = child_with_mandatory_table::table::builder()
        .child_field("Serialized B")
        .try_remote_mandatory_field("Serialized C".to_string())?;

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<child_with_mandatory_table::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(
        deserialized
            .may_get_column_ref::<child_with_mandatory_table::child_field>()
            .map(String::as_str),
        Some("Serialized B")
    );
    assert_eq!(
        deserialized.may_get_column_ref::<child_with_mandatory_table::remote_mandatory_field>(),
        Some(&Some("Serialized C".to_string()))
    );

    Ok(())
}
