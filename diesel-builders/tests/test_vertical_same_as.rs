//! Test case for checking that vertical same-as dependencies
//! work correctly in table inheritance scenarios.

mod shared;

use std::convert::Infallible;

use diesel::prelude::*;
use diesel_builders::prelude::*;

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = parent_table)]
#[table_model(surrogate_key)]
/// A parent table model.
pub struct Parent {
    /// Primary key.
    id: i32,
    /// A field in the parent table.
    parent_field: Option<String>,
    /// Another field in the parent table.
    another_field: String,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
/// Model for a child table that inherits from `parent_table`.
pub struct Child {
    /// Primary key.
    id: i32,
    /// Child specific field.
    #[same_as(parent_table::parent_field, parent_table::another_field)]
    child_field: String,
}

#[test]
fn test_inheritance_vertical_same_as() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS parent_table (
            id INTEGER PRIMARY KEY,
            parent_field TEXT NOT NULL,
            another_field TEXT NOT NULL,
            UNIQUE (id, parent_field),
            UNIQUE (id, another_field)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS child_table (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
            child_field TEXT NOT NULL,
            FOREIGN KEY (id, child_field) REFERENCES parent_table(id, parent_field),
            FOREIGN KEY (id, child_field) REFERENCES parent_table(id, another_field)
        )",
    )
    .execute(&mut conn)?;

    let child = child_table::table::builder()
        .child_field("Child Value")
        .insert(&mut conn)?;

    let parent: Parent = child.ancestor(&mut conn)?;
    assert_eq!(parent.parent_field(), &Some("Child Value".to_owned()));
    assert_eq!(parent.another_field(), "Child Value");

    Ok(())
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = parent_table_checked)]
#[table_model(surrogate_key, error = ParentCheckedError)]
/// A parent table model.
pub struct ParentChecked {
    /// Primary key.
    id: i32,
    /// A field in the parent table.
    parent_field: String,
    /// Another field in the parent table.
    #[infallible]
    another_field: String,
}

#[derive(Debug, thiserror::Error)]
/// Error type for parent table validation.
pub enum ParentCheckedError {
    #[error("Field cannot be empty")]
    /// Error for empty field.
    EmptyField,
}

impl From<Infallible> for ParentCheckedError {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl ValidateColumn<parent_table_checked::parent_field>
    for <parent_table_checked::table as TableExt>::NewValues
{
    type Error = ParentCheckedError;

    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() {
            return Err(ParentCheckedError::EmptyField);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
/// Error type for child table validation.
pub enum ChildCheckedError {
    #[error("Field length exceeds maximum allowed length")]
    /// Error for excessive field length.
    ExcessiveLength,
    /// The parent field error.
    #[error(transparent)]
    ParentFieldError(ParentCheckedError),
}

impl From<ParentCheckedError> for ChildCheckedError {
    fn from(err: ParentCheckedError) -> Self {
        Self::ParentFieldError(err)
    }
}

impl From<Infallible> for ChildCheckedError {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table_checked, error = ChildCheckedError)]
#[diesel(table_name = child_table_checked)]
/// Model for a child table that inherits from `parent_table_checked`.
pub struct ChildChecked {
    #[infallible]
    /// Primary key.
    id: i32,
    /// Child specific field.
    #[same_as(
        parent_table_checked::parent_field,
        parent_table_checked::another_field
    )]
    child_field: String,
}

impl ValidateColumn<child_table_checked::child_field>
    for <child_table_checked::table as TableExt>::NewValues
{
    type Error = ChildCheckedError;

    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.len() > 50 {
            return Err(ChildCheckedError::ExcessiveLength);
        }
        Ok(())
    }
}

#[test]
fn test_inheritance_vertical_same_as_checked() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS parent_table_checked (
            id INTEGER PRIMARY KEY,
            parent_field TEXT NOT NULL CHECK(parent_field <> ''),
            another_field TEXT NOT NULL,
            UNIQUE (id, parent_field),
            UNIQUE (id, another_field)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS child_table_checked (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table_checked(id),
            child_field TEXT NOT NULL CHECK(length(child_field) <= 50),
            FOREIGN KEY (id, child_field) REFERENCES parent_table_checked(id, parent_field),
            FOREIGN KEY (id, child_field) REFERENCES parent_table_checked(id, another_field)
        )",
    )
    .execute(&mut conn)?;

    let mut builder = child_table_checked::table::builder();

    // If we try to set an empty value, it should fail validation from the parent.
    let parent_err = builder.try_child_field_ref("").unwrap_err();
    assert!(matches!(
        parent_err,
        ChildCheckedError::ParentFieldError(ParentCheckedError::EmptyField)
    ));
    assert_eq!(parent_err.to_string(), "Field cannot be empty");

    // If we try to set an excessively long value, it should fail validation from the child.
    let child_err = builder
        .try_child_field_ref("This is a very long string that exceeds the maximum allowed length for the child field.")
        .unwrap_err();
    assert!(matches!(child_err, ChildCheckedError::ExcessiveLength));
    assert_eq!(
        child_err.to_string(),
        "Field length exceeds maximum allowed length"
    );

    let child = child_table_checked::table::builder()
        .try_child_field("Child Value")?
        .insert(&mut conn)?;

    let parent: ParentChecked = child.ancestor(&mut conn)?;
    assert_eq!(parent.parent_field(), "Child Value");

    Ok(())
}
