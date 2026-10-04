//! Rejected discretionary model references preserve complete builder state.

mod shared;

use std::convert::Infallible;

use diesel_builders::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Validation errors for the checked tables.
pub enum ValidationError {
    /// The label exceeds its permitted length.
    #[error("Label exceeds its permitted length")]
    LabelTooLong,
    /// The label is reserved by the parent.
    #[error("Label is reserved by the parent")]
    ReservedLabel,
}

impl From<Infallible> for ValidationError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = checked_parents)]
#[table_model(surrogate_key, error = ValidationError)]
/// Model for the checked parent table.
pub struct CheckedParent {
    /// Primary key.
    id: i32,
    /// A label reserved by the validator for one value.
    label: String,
}

impl ValidateColumn<checked_parents::label> for <checked_parents::table as TableExt>::NewValues {
    type Error = ValidationError;
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value == "deny" {
            Err(ValidationError::ReservedLabel)
        } else if value.len() > 10 {
            Err(ValidationError::LabelTooLong)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = checked_sides)]
#[table_model(surrogate_key, error = ValidationError, foreign_key(parent_id, (checked_parents::id)))]
/// Model for the checked side table.
pub struct CheckedSide {
    /// Primary key.
    id: i32,
    /// Foreign key to the parent table.
    #[infallible]
    parent_id: i32,
    /// A short label.
    label: String,
}

impl ValidateColumn<checked_sides::label> for <checked_sides::table as TableExt>::NewValues {
    type Error = ValidationError;
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.len() > 5 { Err(ValidationError::LabelTooLong) } else { Ok(()) }
    }
}

unique_index!(checked_sides::id, checked_sides::parent_id);
unique_index!(checked_sides::id, checked_sides::label);

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = checked_children)]
#[table_model(ancestors(checked_parents), error = ValidationError)]
/// Model for the checked child table.
pub struct CheckedChild {
    /// Primary key and reference to the parent.
    #[infallible]
    id: i32,
    /// Label propagated from the parent and the discretionary side.
    #[same_as(checked_parents::label)]
    #[same_as(checked_sides::label, discretionary_id)]
    child_label: String,
    /// Optional notes untouched by the reference attempts.
    #[infallible]
    notes: Option<String>,
    /// Reference to the discretionary side.
    #[discretionary(checked_sides)]
    #[infallible]
    discretionary_id: i32,
}

impl ValidateColumn<checked_children::child_label>
    for <checked_children::table as TableExt>::NewValues
{
    type Error = ValidationError;
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.len() > 12 { Err(ValidationError::LabelTooLong) } else { Ok(()) }
    }
}

diesel::allow_tables_to_appear_in_same_query!(checked_parents, checked_sides, checked_children);

#[test]
fn rejected_discretionary_model_ref_preserves_full_state() -> Result<(), Box<dyn std::error::Error>>
{
    let mut conn = shared::establish_connection()?;

    // The DSL cannot express table creation.
    diesel::sql_query(
        "CREATE TABLE checked_parents (
            id INTEGER PRIMARY KEY,
            label TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE checked_sides (
            id INTEGER PRIMARY KEY NOT NULL,
            parent_id INTEGER NOT NULL REFERENCES checked_parents(id),
            label TEXT NOT NULL,
            UNIQUE(id, parent_id),
            UNIQUE(id, label)
        )",
    )
    .execute(&mut conn)?;
    diesel::sql_query(
        "CREATE TABLE checked_children (
            id INTEGER PRIMARY KEY REFERENCES checked_parents(id) ON DELETE CASCADE,
            child_label TEXT NOT NULL,
            notes TEXT,
            discretionary_id INTEGER NOT NULL REFERENCES checked_sides(id)
        )",
    )
    .execute(&mut conn)?;

    diesel::insert_into(checked_parents::table)
        .values((checked_parents::id.eq(3), checked_parents::label.eq("base")))
        .execute(&mut conn)?;
    diesel::insert_into(checked_sides::table)
        .values((
            checked_sides::id.eq(7),
            checked_sides::parent_id.eq(3),
            checked_sides::label.eq("short"),
        ))
        .execute(&mut conn)?;
    diesel::insert_into(checked_sides::table)
        .values((
            checked_sides::id.eq(9),
            checked_sides::parent_id.eq(3),
            checked_sides::label.eq("short"),
        ))
        .execute(&mut conn)?;

    let mut child = checked_children::table::builder()
        .try_child_label("kept")?
        .try_notes("draft".to_owned())?;

    assert_eq!(child.may_get_column::<checked_children::child_label>().as_deref(), Some("kept"),);
    assert_eq!(child.may_get_column::<checked_children::notes>(), Some(Some("draft".into())));
    assert_eq!(child.may_get_column::<checked_children::discretionary_id>(), None);

    let accepted = CheckedSide { id: 7, parent_id: 3, label: "short".into() };
    child.try_discretionary_model_ref(&accepted)?;

    assert_eq!(child.may_get_column::<checked_children::discretionary_id>(), Some(7));
    assert_eq!(child.may_get_column::<checked_children::child_label>().as_deref(), Some("short"),);
    assert_eq!(child.may_get_column::<checked_children::notes>(), Some(Some("draft".into())));

    let saved = child.clone();
    let rejected = CheckedSide { id: 8, parent_id: 4, label: "deny".into() };
    assert_eq!(
        child.try_discretionary_model_ref(&rejected).err(),
        Some(ValidationError::ReservedLabel),
    );

    assert_eq!(child, saved);
    assert_eq!(child.may_get_column::<checked_children::discretionary_id>(), Some(7));
    assert_eq!(child.may_get_column::<checked_children::child_label>().as_deref(), Some("short"),);
    assert_eq!(child.may_get_column::<checked_children::notes>(), Some(Some("draft".into())));

    let sides: i64 = checked_sides::table.count().get_result(&mut conn)?;
    let parents: i64 = checked_parents::table.count().get_result(&mut conn)?;
    assert_eq!(sides, 2);
    assert_eq!(parents, 1);

    let other = CheckedSide { id: 9, parent_id: 3, label: "short".into() };
    child.try_discretionary_model_ref(&other)?;
    assert_eq!(child.may_get_column::<checked_children::discretionary_id>(), Some(9));

    let child = child.insert(&mut conn)?;
    assert_eq!(child.child_label, "short");
    assert_eq!(child.notes.as_deref(), Some("draft"));
    assert_eq!(child.discretionary_id, 9);

    let parent: CheckedParent = child.ancestor::<CheckedParent>(&mut conn)?;
    assert_eq!(parent.id, child.id);
    assert_eq!(parent.label, "short");
    let side = child.discretionary(&mut conn)?;
    assert_eq!(side.id, 9);
    assert_eq!(side.parent_id, 3);
    assert_eq!(side.label, "short");

    let parents: i64 = checked_parents::table.count().get_result(&mut conn)?;
    assert_eq!(parents, 2);

    Ok(())
}
