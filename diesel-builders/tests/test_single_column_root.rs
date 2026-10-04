//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod shared;
use diesel_builders::prelude::*;

#[derive(Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = single_column_root_table)]
#[table_model(surrogate_key, error = SingleColumnRootError)]
/// Model for the `single_column_root_table`.
pub struct SingleColumnRoot {
    /// Primary key.
    id: i32,
    /// A name field.
    #[table_model(default = "")]
    name: String,
}

#[derive(Debug, PartialEq, thiserror::Error)]
/// Errors for `NewSingleColumnRoot` validation.
pub enum SingleColumnRootError {
    /// Name cannot be empty.
    #[error("Name cannot be empty")]
    EmptyName,
}

impl ValidateColumn<single_column_root_table::name>
    for <single_column_root_table::table as TableExt>::NewValues
{
    type Error = SingleColumnRootError;

    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            return Err(SingleColumnRootError::EmptyName);
        }
        Ok(())
    }
}

#[test]
fn test_single_column_root() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS single_column_root_table (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL CHECK(name <> '')
        )",
    )
    .execute(&mut conn)?;

    // Test Root derive - animals table is a root with no ancestors
    let mut builder = single_column_root_table::table::empty_builder();

    let err = builder.try_name_ref("  ").unwrap_err();
    assert_eq!(err, SingleColumnRootError::EmptyName);

    let _row = builder.try_name("Buddy")?.insert(&mut conn)?;

    Ok(())
}

/// Invalid defaults reject construction while explicit checked values remain
/// insertable.
#[test]
fn test_invalid_field_default_is_validated() {
    let mut conn = shared::establish_connection().unwrap();
    diesel::sql_query(
        "CREATE TABLE single_column_root_table (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
    )
    .execute(&mut conn)
    .unwrap();
    assert!(matches!(
        single_column_root_table::table::try_builder(),
        Err(SingleColumnRootError::EmptyName)
    ));
    let valid = single_column_root_table::table::empty_builder()
        .try_name("Validated")
        .unwrap()
        .insert(&mut conn)
        .unwrap();
    assert_eq!(valid.name(), "Validated");
}

#[cfg(feature = "serde")]
#[test]
fn test_deserialization_rejects_invalid_field() {
    let builder = single_column_root_table::table::empty_builder().try_name("Valid").unwrap();
    let encoded = serde_json::to_string(&builder).unwrap();
    let invalid = encoded.replace("Valid", "");
    let result = serde_json::from_str::<TableBuilder<single_column_root_table::table>>(&invalid);
    assert!(result.is_err(), "deserialization exposed an invalid checked builder");
    let decoded: TableBuilder<single_column_root_table::table> =
        serde_json::from_str(&encoded).unwrap();
    let mut conn = shared::establish_connection().unwrap();
    diesel::sql_query(
        "CREATE TABLE single_column_root_table (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
    )
    .execute(&mut conn)
    .unwrap();
    let row = decoded.insert(&mut conn).unwrap();
    assert_eq!(row.name(), "Valid");
}
