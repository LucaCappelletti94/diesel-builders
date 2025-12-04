//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod common;

use common::*;
use diesel::prelude::*;
use diesel_builders::prelude::*;

#[test]
fn test_simple_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    // Test Root derive - animals table is a root with no ancestors
    let mut builder = animals::table::builder();

    // Test MayGetColumn derive - optional fields start as None
    assert_eq!(builder.may_get_column::<animals::name>(), None);
    assert_eq!(builder.may_get_column::<animals::description>(), None);

    // Test generated TrySetAnimalsName helper trait - fallible setter by reference
    builder.try_name_ref("Max")?;

    // Test MayGetColumn derive - verifying field is set after mutation
    assert_eq!(
        builder
            .may_get_column::<animals::name>()
            .map(String::as_str),
        Some("Max")
    );
    assert_eq!(builder.may_get_column::<animals::description>(), None);

    let animal = builder.insert(&mut conn)?;

    assert_eq!(animal.name, "Max");
    assert_eq!(animal.description, None);

    // Test GetColumn derive - type-safe column access on models
    assert_eq!(animal.name(), "Max");
    assert!(animal.description().is_none());

    // Test TableModel derive - primary key access
    assert!(animal.id() > &0);

    // Test with description set to Some value - using generated helper traits
    let animal_with_desc = animals::table::builder()
        .try_name("Buddy")?
        .try_description("A friendly dog".to_owned())?
        .insert(&mut conn)?;

    assert_eq!(
        animal_with_desc.description.as_deref(),
        Some("A friendly dog")
    );

    // Test with description explicitly set to None (NULL in database)
    let animal_no_desc = animals::table::builder()
        .try_name("Whiskers")?
        .try_description(None)?
        .insert(&mut conn)?;

    assert_eq!(animal_no_desc.description, None);

    // We attempt to query the inserted animal to ensure everything worked correctly.
    let queried_animal: Animal = animals::table
        .filter(animals::id.eq(animal.id))
        .first(&mut conn)?;
    assert_eq!(animal, queried_animal);

    // Test chained builder pattern with GetColumn derive
    let another_animal = animals::table::builder()
        .try_name("Charlie")?
        .insert(&mut conn)?;

    // Test GetColumn derive on multiple fields
    assert_eq!(another_animal.name(), "Charlie");

    // Test TableModel derive - verifying unique primary keys
    assert_ne!(animal.id(), another_animal.id());

    Ok(())
}

#[test]
fn test_empty_name_rejected() {
    let result = animals::table::builder().try_name(String::new());
    assert_eq!(result.unwrap_err(), NewAnimalError::NameEmpty);
}

#[test]
fn test_whitespace_only_name_rejected() {
    let result = animals::table::builder().try_name("   ".to_string());
    assert_eq!(result.unwrap_err(), NewAnimalError::NameEmpty);
}

#[test]
fn test_name_too_long_rejected() {
    let long_name = "a".repeat(101);
    let result = animals::table::builder().try_name(long_name);
    assert_eq!(result.unwrap_err(), NewAnimalError::NameTooLong);
}

#[test]
fn test_empty_description_rejected() {
    let result = animals::table::builder().try_description(Some(String::new()));
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionEmpty);
}

#[test]
fn test_whitespace_only_description_rejected() {
    let result = animals::table::builder().try_description(Some("   ".to_string()));
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionEmpty);
}

#[test]
fn test_description_too_long_rejected() {
    let long_desc = "a".repeat(501);
    let result = animals::table::builder().try_description(Some(long_desc));
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionTooLong);
}

#[test]
fn test_none_description_allowed() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = animals::table::builder();
    builder.try_description_ref(None)?;
    assert_eq!(
        builder.may_get_column::<animals::description>(),
        Some(&None)
    );
    Ok(())
}

#[test]
fn test_insert_fails_when_table_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Intentionally do NOT create the animals table

    let result = animals::table::builder().try_name("Max")?.insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Diesel(_)
    ));

    Ok(())
}
