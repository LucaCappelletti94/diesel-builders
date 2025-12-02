//! Tests for the `Display` implementation of `NewAnimalError`.

mod common;

use common::NewAnimalError;

#[test]
fn new_animal_error_display() {
    assert_eq!(
        format!("{}", NewAnimalError::NameEmpty),
        "Animal name cannot be empty"
    );
    assert_eq!(
        format!("{}", NewAnimalError::NameTooLong),
        "Animal name cannot exceed 100 characters"
    );
    assert_eq!(
        format!("{}", NewAnimalError::DescriptionEmpty),
        "Animal description cannot be empty when provided"
    );
    assert_eq!(
        format!("{}", NewAnimalError::DescriptionTooLong),
        "Animal description cannot exceed 500 characters"
    );
}
