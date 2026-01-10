//! Tests for `BuilderError` Display and Error trait implementations.

use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind};
use diesel_builders::{BuilderError, IncompleteBuilderError};
use std::{error::Error, num::ParseIntError};

#[test]
fn test_builder_error_diesel_display() {
    let diesel_error = diesel::result::Error::NotFound;
    let builder_error: BuilderError<diesel::result::Error> = BuilderError::Diesel(diesel_error);

    let display_string = format!("{builder_error}");
    assert_eq!(display_string, "Diesel error: Record not found");
}

#[test]
fn test_builder_error_incomplete_display() {
    let incomplete_error = IncompleteBuilderError::MissingMandatoryTriangularField("c_id");
    let builder_error: BuilderError<IncompleteBuilderError> =
        BuilderError::Incomplete(incomplete_error);

    let display_string = format!("{builder_error}");
    assert_eq!(
        display_string,
        "Missing mandatory triangular builder field: `c_id`"
    );
}

#[test]
fn test_builder_error_validation_display() {
    let validation_error: ParseIntError = "abc".parse::<i32>().unwrap_err();
    let builder_error = BuilderError::Validation(validation_error);

    let display_string = format!("{builder_error}");
    assert_eq!(
        display_string,
        "Validation error: invalid digit found in string"
    );
}

#[test]
fn test_builder_error_diesel_source() {
    let diesel_error = diesel::result::Error::NotFound;
    let builder_error: BuilderError<ParseIntError> = BuilderError::Diesel(diesel_error);

    let source = builder_error.source();
    assert!(source.is_some());
    assert_eq!(source.unwrap().to_string(), "Record not found");
}

#[test]
fn test_builder_error_incomplete_source() {
    let incomplete_error = IncompleteBuilderError::MissingMandatoryTriangularField("c_id");
    let builder_error: BuilderError<ParseIntError> = BuilderError::Incomplete(incomplete_error);

    let source = builder_error.source();
    assert!(source.is_some());
    assert_eq!(
        source.unwrap().to_string(),
        "Missing mandatory triangular builder field: `c_id`"
    );
}

#[test]
fn test_builder_error_validation_source() {
    let validation_error = "abc".parse::<i32>().unwrap_err();
    let builder_error = BuilderError::Validation(validation_error);

    let source = builder_error.source();
    assert!(source.is_some());
    assert_eq!(source.unwrap().to_string(), "invalid digit found in string");
}

#[test]
fn test_incomplete_builder_error_display() {
    let error = IncompleteBuilderError::MissingMandatoryTriangularField("c_id");
    let display_string = format!("{error}");
    assert_eq!(
        display_string,
        "Missing mandatory triangular builder field: `c_id`"
    );

    let error = IncompleteBuilderError::MissingMandatoryField("name");
    let display_string = format!("{error}");
    assert_eq!(display_string, "Missing mandatory field: `name`");
}

#[test]
fn test_incomplete_builder_error_database_error_information() {
    let error = IncompleteBuilderError::MissingMandatoryTriangularField("c_id");
    assert_eq!(
        error.message(),
        "Missing mandatory triangular builder field"
    );
    assert_eq!(error.details(), None);
    assert_eq!(error.hint(), None);
    assert_eq!(error.table_name(), None);
    assert_eq!(error.column_name(), Some("c_id"));
    assert_eq!(error.constraint_name(), None);
    assert_eq!(error.statement_position(), None);

    let error = IncompleteBuilderError::MissingMandatoryField("name");
    assert_eq!(error.message(), "Missing mandatory field");
    assert_eq!(error.details(), None);
    assert_eq!(error.hint(), None);
    assert_eq!(error.table_name(), None);
    assert_eq!(error.column_name(), Some("name"));
    assert_eq!(error.constraint_name(), None);
    assert_eq!(error.statement_position(), None);
}

#[test]
fn test_from_diesel_error() {
    let diesel_error = diesel::result::Error::NotFound;
    let builder_error: BuilderError<ParseIntError> = diesel_error.into();
    assert!(matches!(builder_error, BuilderError::Diesel(_)));
}

#[test]
fn test_from_incomplete_builder_error() {
    let incomplete_error = IncompleteBuilderError::MissingMandatoryField("name");
    let builder_error: BuilderError<ParseIntError> = incomplete_error.into();
    assert!(matches!(builder_error, BuilderError::Incomplete(_)));
}

#[test]
fn test_builder_error_from_diesel_error_conversion() {
    // Create a mock DatabaseErrorInformation
    #[derive(Debug)]
    struct MockError;
    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "mock error")
        }
    }
    impl std::error::Error for MockError {}
    impl DatabaseErrorInformation for MockError {
        fn message(&self) -> &str {
            "mock message"
        }
        fn details(&self) -> Option<&str> {
            Some("mock details")
        }
        fn hint(&self) -> Option<&str> {
            Some("mock hint")
        }
        fn table_name(&self) -> Option<&str> {
            Some("mock_table")
        }
        fn column_name(&self) -> Option<&str> {
            Some("mock_column")
        }
        fn constraint_name(&self) -> Option<&str> {
            Some("mock_constraint")
        }
        fn statement_position(&self) -> Option<i32> {
            Some(42)
        }
    }

    let mock_error = MockError;
    let builder_error = BuilderError::<MockError>::Validation(mock_error);

    // Convert back to diesel error
    let diesel_error: diesel::result::Error = builder_error.into();
    assert!(matches!(
        diesel_error,
        diesel::result::Error::DatabaseError(_, _)
    ));

    if let diesel::result::Error::DatabaseError(kind, info) = diesel_error {
        assert_eq!(kind, DatabaseErrorKind::CheckViolation);
        assert_eq!(info.message(), "mock message");
        assert_eq!(info.details(), Some("mock details"));
        assert_eq!(info.hint(), Some("mock hint"));
        assert_eq!(info.table_name(), Some("mock_table"));
        assert_eq!(info.column_name(), Some("mock_column"));
        assert_eq!(info.constraint_name(), Some("mock_constraint"));
        assert_eq!(info.statement_position(), Some(42));
    }
}

#[test]
fn test_builder_error_debug() {
    let diesel_error = diesel::result::Error::NotFound;
    let builder_error: BuilderError<ParseIntError> = BuilderError::Diesel(diesel_error);
    let debug_string = format!("{builder_error:?}");
    assert!(debug_string.contains("Diesel"));

    let incomplete_error = IncompleteBuilderError::MissingMandatoryField("name");
    let builder_error: BuilderError<ParseIntError> = BuilderError::Incomplete(incomplete_error);
    let debug_string = format!("{builder_error:?}");
    assert!(debug_string.contains("Incomplete"));

    let validation_error: ParseIntError = "abc".parse::<i32>().unwrap_err();
    let builder_error = BuilderError::Validation(validation_error);
    let debug_string = format!("{builder_error:?}");
    assert!(debug_string.contains("Validation"));
}

#[test]
fn test_incomplete_builder_error_debug() {
    let error = IncompleteBuilderError::MissingMandatoryTriangularField("c_id");
    let debug_string = format!("{error:?}");
    assert!(debug_string.contains("MissingMandatoryTriangularField"));

    let error = IncompleteBuilderError::MissingMandatoryField("name");
    let debug_string = format!("{error:?}");
    assert!(debug_string.contains("MissingMandatoryField"));
}

#[test]
fn test_incomplete_builder_error_partial_eq() {
    let error1 = IncompleteBuilderError::MissingMandatoryField("name");
    let error2 = IncompleteBuilderError::MissingMandatoryField("name");
    let error3 = IncompleteBuilderError::MissingMandatoryField("other");

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);

    let error4 = IncompleteBuilderError::MissingMandatoryTriangularField("name");
    assert_ne!(error1, error4);
}

#[test]
fn test_incomplete_builder_error_clone() {
    let error = IncompleteBuilderError::MissingMandatoryField("name");
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

#[test]
fn test_incomplete_builder_error_copy() {
    let error1 = IncompleteBuilderError::MissingMandatoryField("name");
    let error2 = error1; // Copy
    assert_eq!(error1, error2);
}

#[test]
fn test_incomplete_builder_error_hash() {
    use std::collections::HashSet;

    let error1 = IncompleteBuilderError::MissingMandatoryField("name");
    let error2 = IncompleteBuilderError::MissingMandatoryField("name");
    let error3 = IncompleteBuilderError::MissingMandatoryField("other");

    let mut set = HashSet::new();
    set.insert(error1);
    set.insert(error2);
    set.insert(error3);

    assert_eq!(set.len(), 2); // error1 and error2 should be the same, error3 different
}
