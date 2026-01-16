//! Submodule to test a discretionary triangular relation between tables.

mod shared;
mod shared_triangular;
use std::convert::Infallible;

use diesel::prelude::*;
use diesel_builders::prelude::*;
use shared_triangular::*;

// Table B models
#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(error = ErrorB, ancestors = shared_triangular::parent_table)]
#[diesel(table_name = child_with_satellite_table)]
/// Model for table B.
pub struct ChildWithDiscretionary {
    #[infallible]
    /// Primary key.
    #[same_as(shared_triangular::satellite_table::parent_id)]
    id: i32,
    #[infallible]
    #[discretionary(shared_triangular::satellite_table)]
    /// Foreign key to discretionary table.
    discretionary_id: i32,
    #[infallible]
    /// Some other column in the child table.
    child_field: String,
    /// The remote `field` value from discretionary table that B references via
    /// `discretionary_id`.
    #[same_as(satellite_table::field)]
    #[table_model(default = "Some default value")]
    remote_field: Option<String>,
    /// The remote `parent_id` value from discretionary table that B references
    /// via `discretionary_id`.
    #[infallible]
    #[same_as(shared_triangular::satellite_table::another_field)]
    another_remote_column: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = simple_child_with_satellite_table)]
/// Model for simple child table with discretionary triangular relation.
pub struct SimpleChildWithDiscretionary {
    #[same_as(satellite_table::parent_id)]
    /// Primary key.
    id: i32,
    #[discretionary(satellite_table)]
    /// Foreign key to table A.
    discretionary_id: i32,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
/// Errors for `NewChildWithDiscretionary` validation.
pub enum ErrorB {
    /// `remote_field` cannot be empty.
    #[error("`remote_field` cannot be empty")]
    EmptyRemoteColumnC,
}

impl From<Infallible> for ErrorB {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl ValidateColumn<child_with_satellite_table::remote_field>
    for <child_with_satellite_table::table as TableExt>::NewValues
{
    type Error = ErrorB;

    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(ErrorB::EmptyRemoteColumnC);
        }
        Ok(())
    }
}

#[test]
fn test_discretionary_triangular_relation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_triangular::setup_triangular_tables(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE child_with_satellite_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            discretionary_id INTEGER NOT NULL REFERENCES satellite_table(id),
            child_field TEXT NOT NULL,
            remote_field TEXT CHECK (remote_field <> ''),
            another_remote_column TEXT,
			FOREIGN KEY (discretionary_id, remote_field) REFERENCES satellite_table(id, field),
            FOREIGN KEY (discretionary_id, another_remote_column) REFERENCES satellite_table(id, another_field)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder().parent_field("Value A").insert(&mut conn)?;

    assert_eq!(parent.parent_field(), "Value A");

    // Insert into table C (references A)
    let discretionary = satellite_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .field("Value C")
        .insert(&mut conn)?;

    assert_eq!(discretionary.field(), "Value C");
    assert_eq!(*discretionary.parent_id(), parent.get_column::<parent_table::id>());

    let mut discretionary_builder = satellite_table::table::builder();
    discretionary_builder.field_ref("Value C for B");

    // Insert into table B (extends C and references A)
    // The discretionary triangular relation means we can set the C builder or
    // reference an existing C model Using generated trait methods like
    // try_discretionary_ref for type-safe builders
    let mut child_builder =
        child_with_satellite_table::table::builder().parent_field("Value A for B");

    let saved_child_builder = child_builder.clone();

    assert_eq!(
        child_builder.try_discretionary_ref(satellite_table::table::builder().field(String::new())),
        Err(ErrorB::EmptyRemoteColumnC)
    );

    // Since the operation has failed, the preliminary state of the builder should
    // have remained unchanged.
    assert_eq!(child_builder, saved_child_builder);

    child_builder
        .child_field_ref("Value B")
        .try_discretionary_ref(discretionary_builder.clone())?;

    // Debug formatting test
    let _formatted = format!("{child_builder:?}");

    let child = child_builder
        .try_discretionary(discretionary_builder)?
        .try_another_remote_column("After setting discretionary".to_owned())?
        .another_remote_column("After setting discretionary".to_owned())
        .insert(&mut conn)
        .unwrap();

    let associated_parent: Parent = child.ancestor::<Parent>(&mut conn)?;
    assert_eq!(associated_parent.parent_field(), "Value A for B");

    // We can also reference an existing model using the _model variant
    // Example: triangular_b_builder.discretionary_id_model_ref(&c) would reference
    // the existing c model

    let associated_discretionary: Satellite = child.discretionary(&mut conn)?;
    assert_eq!(
        associated_discretionary.another_field().as_deref(),
        Some("After setting discretionary")
    );
    assert_eq!(
        *associated_discretionary.parent_id(),
        child.get_column::<child_with_satellite_table::id>()
    );
    assert_eq!(
        *associated_discretionary.parent_id(),
        associated_parent.get_column::<parent_table::id>()
    );

    let independent_child = child_with_satellite_table::table::builder()
        .parent_field("Independent A for B")
        .child_field("Independent B")
        .try_discretionary_model(&discretionary)?
        .insert(&mut conn)?;

    assert_eq!(independent_child.child_field(), "Independent B");
    assert_eq!(independent_child.remote_field.as_deref(), Some("Value C"));
    assert_ne!(
        independent_child.get_column::<child_with_satellite_table::id>(),
        child.get_column::<child_with_satellite_table::id>()
    );
    assert_ne!(
        independent_child.get_column::<child_with_satellite_table::id>(),
        child.get_column::<child_with_satellite_table::id>()
    );
    assert_ne!(
        independent_child.get_column::<child_with_satellite_table::id>(),
        *discretionary.parent_id()
    );

    // Test iter_foreign_keys with composite index (satellite_table::id,
    // satellite_table::field)
    let refs: Vec<_> =
        child.iter_foreign_keys::<(satellite_table::id, satellite_table::field)>().collect();
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(&(child.discretionary_id(), child.remote_field.as_ref())));

    let refs_independent: Vec<_> = independent_child
        .iter_foreign_keys::<(satellite_table::id, satellite_table::field)>()
        .collect();
    assert_eq!(refs_independent.len(), 1);
    assert!(refs_independent.contains(&(
        independent_child.discretionary_id(),
        independent_child.remote_field.as_ref()
    )));

    Ok(())
}

#[test]
fn test_discretionary_triangular_relation_simple() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    setup_triangular_tables(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE simple_child_with_satellite_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            discretionary_id INTEGER NOT NULL REFERENCES satellite_table(id),
			FOREIGN KEY (discretionary_id, id) REFERENCES satellite_table(id, parent_id)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder().parent_field("Value A").insert(&mut conn).unwrap();

    assert_eq!(parent.parent_field(), "Value A");

    // Insert into table C (references A)
    let discretionary = satellite_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .field("Value C")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(discretionary.field(), "Value C");
    assert_eq!(*discretionary.parent_id(), parent.get_column::<parent_table::id>());

    let mut discretionary_builder =
        satellite_table::table::builder().another_field("Original another remote field".to_owned());
    discretionary_builder.field_ref("Value C");

    // Insert into table B (extends C and references A)
    // The discretionary triangular relation means B's parent_id should
    // automatically match C's parent_id when we only set C's columns
    // Using generated trait methods like try_discretionary_ref for type-safe
    // builders
    let mut child_builder = simple_child_with_satellite_table::table::builder()
        .parent_field("Value A for B")
        .discretionary_model(&discretionary)
        .try_discretionary_model(&discretionary)?;

    let saved_child_builder = child_builder.clone();

    // Since the operation has failed, the preliminary state of the builder should
    // have remained unchanged.
    assert_eq!(child_builder, saved_child_builder);

    child_builder.try_discretionary_ref(discretionary_builder.clone())?;

    // Using the generated trait method for more ergonomic code
    let child = child_builder.try_discretionary(discretionary_builder)?.insert(&mut conn).unwrap();

    let associated_parent: Parent = child.ancestor::<Parent>(&mut conn)?;
    assert_eq!(associated_parent.parent_field(), "Value A for B");

    let associated_discretionary: Satellite = child.discretionary(&mut conn)?;
    assert_eq!(associated_discretionary.field(), "Value C");
    assert_eq!(
        associated_discretionary.another_field().as_deref(),
        Some("Original another remote field")
    );
    assert_eq!(
        *associated_discretionary.parent_id(),
        child.get_column::<simple_child_with_satellite_table::id>()
    );
    assert_eq!(
        *associated_discretionary.parent_id(),
        associated_parent.get_column::<parent_table::id>()
    );

    // Test iter_foreign_keys with composite index (satellite_table::id,
    // satellite_table::parent_id)
    let refs: Vec<_> =
        child.iter_foreign_keys::<(satellite_table::id, satellite_table::parent_id)>().collect();
    assert_eq!(refs.len(), 1);
    assert!(refs.contains(&(
        &associated_discretionary.get_column::<satellite_table::id>(),
        associated_discretionary.parent_id()
    )));

    Ok(())
}
