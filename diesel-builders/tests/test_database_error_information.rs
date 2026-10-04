//! A rejected multi-column record forwards the inner `ValidationError`'s
//! database-error information through `EitherValidationError` unchanged.

use diesel_builders::{BuilderError, SetColumn, TableBuilderBundle, prelude::*};
use validation_errors::ValidationError;

/// A record with two columns independently validated as non-empty.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = named_pairs)]
#[table_model(surrogate_key, error = ValidationError)]
pub struct NamedPair {
    /// Primary key.
    id: i32,
    /// The first required label.
    first: String,
    /// The second required label.
    second: String,
}

impl ValidateColumn<named_pairs::first> for <named_pairs::table as TableExt>::NewValues {
    type Error = ValidationError;
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() { Err(ValidationError::empty("named_pairs", "first")) } else { Ok(()) }
    }
}

impl ValidateColumn<named_pairs::second> for <named_pairs::table as TableExt>::NewValues {
    type Error = ValidationError;
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.is_empty() { Err(ValidationError::empty("named_pairs", "second")) } else { Ok(()) }
    }
}

fn raw_values(first: &str, second: &str) -> <named_pairs::table as TableExt>::NewValues {
    let mut values = named_pairs::table::empty_new_values();
    SetColumn::<named_pairs::first>::set_column(&mut values, first.to_owned());
    SetColumn::<named_pairs::second>::set_column(&mut values, second.to_owned());
    values
}

#[test]
fn rejected_second_column_forwards_database_error_information() {
    let values = raw_values("Ada", "");
    let (_, error) = TableBuilderBundle::<named_pairs::table>::try_from_values(values).unwrap_err();
    let builder_error: BuilderError<_> = BuilderError::Validation(error);
    let diesel_error: diesel::result::Error = builder_error.into();
    assert!(
        matches!(diesel_error, diesel::result::Error::DatabaseError(..)),
        "expected a DatabaseError variant"
    );
    let diesel::result::Error::DatabaseError(kind, info) = diesel_error else { return };
    assert_eq!(kind, diesel::result::DatabaseErrorKind::CheckViolation);
    assert_eq!(info.table_name(), Some("named_pairs"));
    assert_eq!(info.column_name(), Some("second"));
    assert_eq!(info.message(), "Field must not be empty");
    assert_eq!(info.details(), None);
    assert_eq!(info.hint(), None);
    assert_eq!(info.constraint_name(), None);
    assert_eq!(info.statement_position(), None);
}

#[test]
fn rejected_first_column_forwards_database_error_information() {
    let values = raw_values("", "Grace");
    let (_, error) = TableBuilderBundle::<named_pairs::table>::try_from_values(values).unwrap_err();
    let builder_error: BuilderError<_> = BuilderError::Validation(error);
    let diesel_error: diesel::result::Error = builder_error.into();
    assert!(
        matches!(diesel_error, diesel::result::Error::DatabaseError(..)),
        "expected a DatabaseError variant"
    );
    let diesel::result::Error::DatabaseError(_, info) = diesel_error else { return };
    assert_eq!(info.table_name(), Some("named_pairs"));
    assert_eq!(info.column_name(), Some("first"));
}
