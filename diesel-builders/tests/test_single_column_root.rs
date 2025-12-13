//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod shared;
use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = single_column_root_table)]
#[table_model(surrogate_key, error = SingleColumnRootError)]
/// Model for the `single_column_root_table`.
pub struct SingleColumnRoot {
    /// Primary key.
    id: i32,
    /// A name field.
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
/// Errors for `NewSingleColumnRoot` validation.
pub enum SingleColumnRootError {
    /// Name cannot be empty.
    #[error("Name cannot be empty")]
    EmptyName,
}

impl From<std::convert::Infallible> for SingleColumnRootError {
    fn from(inf: std::convert::Infallible) -> Self {
        match inf {}
    }
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
    let mut builder = single_column_root_table::table::builder();

    let err = builder.try_name_ref("  ").unwrap_err();
    assert_eq!(err, SingleColumnRootError::EmptyName);

    let _row = builder.try_name("Buddy")?.insert(&mut conn)?;

    Ok(())
}
