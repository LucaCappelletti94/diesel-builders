//! Tests for `BuilderError` Display and Error trait implementations.

mod common;

use common::NewAnimalError;
use diesel_builders::{BuilderError, IncompleteBuilderError};
use std::error::Error;

#[test]
fn test_builder_error_diesel_display() {
    let diesel_error = diesel::result::Error::NotFound;
    let builder_error: BuilderError<diesel::result::Error> = BuilderError::Diesel(diesel_error);

    let display_string = format!("{builder_error}");
    assert_eq!(display_string, "Diesel error: Record not found");
}

#[test]
fn test_builder_error_incomplete_display() {
    let incomplete_error = IncompleteBuilderError::MissingMandatoryTriangularFields;
    let builder_error: BuilderError<IncompleteBuilderError> =
        BuilderError::Incomplete(incomplete_error);

    let display_string = format!("{builder_error}");
    assert_eq!(
        display_string,
        "Incomplete builder error: Not all mandatory associated builders have been set"
    );
}

#[test]
fn test_builder_error_validation_display() {
    let validation_error = NewAnimalError::NameEmpty;
    let builder_error = BuilderError::Validation(validation_error);

    let display_string = format!("{builder_error}");
    assert_eq!(
        display_string,
        "Validation error: Animal name cannot be empty"
    );
}

#[test]
fn test_builder_error_diesel_source() {
    let diesel_error = diesel::result::Error::NotFound;
    let builder_error: BuilderError<NewAnimalError> = BuilderError::Diesel(diesel_error);

    let source = builder_error.source();
    assert!(source.is_some());
    assert_eq!(source.unwrap().to_string(), "Record not found");
}

#[test]
fn test_builder_error_incomplete_source() {
    let incomplete_error = IncompleteBuilderError::MissingMandatoryTriangularFields;
    let builder_error: BuilderError<NewAnimalError> = BuilderError::Incomplete(incomplete_error);

    let source = builder_error.source();
    assert!(source.is_some());
    assert_eq!(
        source.unwrap().to_string(),
        "Not all mandatory associated builders have been set"
    );
}

#[test]
fn test_builder_error_validation_source() {
    let validation_error = NewAnimalError::NameTooLong;
    let builder_error = BuilderError::Validation(validation_error);

    let source = builder_error.source();
    assert!(source.is_some());
    assert_eq!(
        source.unwrap().to_string(),
        "Animal name cannot exceed 100 characters"
    );
}

#[test]
fn test_incomplete_builder_error_display() {
    let error = IncompleteBuilderError::MissingMandatoryTriangularFields;
    let display_string = format!("{error}");
    assert_eq!(
        display_string,
        "Not all mandatory associated builders have been set"
    );
}
