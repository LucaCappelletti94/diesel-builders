//! Tests for `BuilderError` Display and Error trait implementations.

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
        "Incomplete builder error: Missing mandatory triangular builder field: c_id"
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
        "Missing mandatory triangular builder field: c_id"
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
        "Missing mandatory triangular builder field: c_id"
    );
}
