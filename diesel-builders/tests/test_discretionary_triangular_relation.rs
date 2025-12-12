//! Submodule to test a discretionary triangular relation between tables.
//!
//! This test sets up three tables: Parent, Child, and Discretionary. Child extends Discretionary, and Discretionary contains
//! a column that references Parent, and Child has a column that references Discretionary, forming a
//! triangular relationship. The test verifies that inserts and queries work
//! correctly through this relationship.
//!
//! Specifically, the relationship is discretionary, that is the foreign key
//! from Discretionary to Parent is NOT referenced in Child using a same-as relationship, which means
//! that the Discretionary record associated with a Child record may reference the same Parent record
//! as Child does, but it is not required to and the user can choose to set it or
//! not. Additionally, there exist a same-as relationship between Child and Discretionary on
//! another column, which means that when setting the builder for the Discretionary record
//! in the Child builder, that column value needs to be set, and when it is not set
//! by setting the associated Discretionary builder, it must be set manually in Child.

mod shared;
mod shared_triangular;
use shared_triangular::*;

use std::convert::Infallible;

use diesel::prelude::*;
use diesel_builders::prelude::*;

// Allow tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(
    child_with_discretionary_table,
    parent_table,
    discretionary_table
);

// Table B models
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(error = ErrorB, ancestors = parent_table)]
#[diesel(table_name = child_with_discretionary_table)]
/// Model for table B.
pub struct ChildWithDiscretionary {
    #[infallible]
    /// Primary key.
    id: i32,
    #[infallible]
    #[discretionary]
    /// Foreign key to discretionary table.
    discretionary_id: i32,
    #[infallible]
    /// Some other column in the child table.
    child_field: String,
    /// The remote `discretionary_field` value from discretionary table that B references via `discretionary_id`.
    remote_discretionary_field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, thiserror::Error)]
/// Errors for `NewChildWithDiscretionary` validation.
pub enum ErrorB {
    /// `remote_discretionary_field` cannot be empty.
    #[error("`remote_discretionary_field` cannot be empty")]
    EmptyRemoteColumnC,
}

impl From<Infallible> for ErrorB {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl TrySetColumn<child_with_discretionary_table::remote_discretionary_field>
    for <child_with_discretionary_table::table as TableExt>::NewValues
{
    type Error = ErrorB;

    fn try_set_column(&mut self, value: Option<String>) -> Result<&mut Self, Self::Error> {
        if let Some(ref v) = value
            && v.trim().is_empty()
        {
            return Err(ErrorB::EmptyRemoteColumnC);
        }
        self.set_column_unchecked::<child_with_discretionary_table::remote_discretionary_field>(
            value,
        );
        Ok(self)
    }
}

// Declare singleton foreign key for child_with_discretionary_table::discretionary_id to discretionary_table
fpk!(child_with_discretionary_table::discretionary_id -> discretionary_table);

// Define foreign key relationship using SQL-like syntax
// B's (discretionary_id, remote_discretionary_field) references C's (id, discretionary_field)
fk!((child_with_discretionary_table::discretionary_id, child_with_discretionary_table::remote_discretionary_field) -> (discretionary_table::id, discretionary_table::discretionary_field));

// This is the key part: B's discretionary_id must match C's id, and C's parent_id must match A's
// id. We express that B's discretionary_id is horizontally the same as C's parent_id, which in
// turn is the same as A's id.
impl diesel_builders::HorizontalKey for child_with_discretionary_table::discretionary_id {
    // HostColumns are columns in child_with_discretionary_table (the same table) that relate to this key
    // In this case, there are no other columns in child_with_discretionary_table that need to match
    // Actually, we need to think about this differently...
    type HostColumns = (
        child_with_discretionary_table::id,
        child_with_discretionary_table::remote_discretionary_field,
    );
    type ForeignColumns = (
        discretionary_table::parent_id,
        discretionary_table::discretionary_field,
    );
}

#[test]
fn test_discretionary_triangular_relation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_triangular::setup_triangular_tables(&mut conn)?;

    // Create table B (extends C and also references A)
    diesel::sql_query(
        "CREATE TABLE child_with_discretionary_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            discretionary_id INTEGER NOT NULL REFERENCES discretionary_table(id),
            child_field TEXT NOT NULL,
            remote_discretionary_field TEXT CHECK (remote_discretionary_field <> ''),
			FOREIGN KEY (discretionary_id, remote_discretionary_field) REFERENCES discretionary_table(id, discretionary_field)
        )",
    )
    .execute(&mut conn)?;

    // Insert into table A
    let parent = parent_table::table::builder()
        .parent_field("Value A")
        .insert(&mut conn)
        .unwrap();

    assert_eq!(parent.get_column::<parent_table::parent_field>(), "Value A");

    // Insert into table C (references A)
    let discretionary = discretionary_table::table::builder()
        .parent_id(parent.get_column::<parent_table::id>())
        .discretionary_field(Some("Value C".to_owned()))
        .insert(&mut conn)
        .unwrap();

    assert_eq!(
        discretionary
            .get_column::<discretionary_table::discretionary_field>()
            .as_deref(),
        Some("Value C")
    );
    assert_eq!(
        discretionary.get_column::<discretionary_table::parent_id>(),
        parent.get_column::<parent_table::id>()
    );

    let mut discretionary_builder = discretionary_table::table::builder();
    discretionary_builder.discretionary_field_ref(Some("Value C for B".to_owned()));

    // Insert into table B (extends C and references A)
    // The discretionary triangular relation means we can set the C builder or reference an existing C model
    // Using generated trait methods like try_discretionary_ref for type-safe builders
    let mut child_builder = child_with_discretionary_table::table::builder();

    assert!(matches!(
        child_builder.try_discretionary_ref(
            discretionary_table::table::builder().discretionary_field(String::new())
        ),
        Err(ErrorB::EmptyRemoteColumnC)
    ));

    child_builder
        .parent_field_ref("Value A for B")
        .child_field_ref("Value B")
        .try_discretionary_ref(discretionary_builder.clone())?;

    // Debug formatting test
    let _formatted = format!("{child_builder:?}");

    let child = child_builder
        .try_discretionary(discretionary_builder)?
        .insert(&mut conn)?;

    let associated_parent: Parent = child.id_fk(&mut conn)?;
    assert_eq!(
        associated_parent.get_column::<parent_table::parent_field>(),
        "Value A for B"
    );

    // We can also reference an existing model using the _model variant
    // Example: triangular_b_builder.discretionary_id_model_ref(&c) would reference the existing c model

    let associated_discretionary: Discretionary = child.discretionary(&mut conn)?;
    assert_eq!(
        associated_discretionary
            .get_column::<discretionary_table::discretionary_field>()
            .as_deref(),
        Some("Value C for B")
    );
    assert_eq!(
        associated_discretionary.get_column::<discretionary_table::parent_id>(),
        child.get_column::<child_with_discretionary_table::id>()
    );
    assert_eq!(
        associated_discretionary.get_column::<discretionary_table::parent_id>(),
        associated_parent.get_column::<parent_table::id>()
    );

    let independent_child = child_with_discretionary_table::table::builder()
        .parent_field("Independent A for B")
        .child_field("Independent B")
        .try_discretionary_model(&discretionary)?
        .insert(&mut conn)
        .unwrap();

    assert_eq!(
        independent_child.get_column::<child_with_discretionary_table::child_field>(),
        "Independent B"
    );
    assert_eq!(
        independent_child.remote_discretionary_field.as_deref(),
        Some("Value C")
    );
    assert_ne!(
        independent_child.get_column::<child_with_discretionary_table::id>(),
        child.get_column::<child_with_discretionary_table::id>()
    );
    assert_ne!(
        independent_child.get_column::<child_with_discretionary_table::id>(),
        child.get_column::<child_with_discretionary_table::id>()
    );
    assert_ne!(
        independent_child.get_column::<child_with_discretionary_table::id>(),
        discretionary.get_column::<discretionary_table::parent_id>()
    );

    Ok(())
}
