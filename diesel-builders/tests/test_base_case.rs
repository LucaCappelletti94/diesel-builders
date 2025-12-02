//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod common;

use common::{Animal, CREATE_ANIMALS_TABLE, NewAnimalError, animals};
use diesel::prelude::*;
use diesel_builders::prelude::*;

#[test]
fn test_simple_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    let mut builder = animals::table::builder();

    assert_eq!(builder.may_get_column::<animals::name>(), None);
    assert_eq!(builder.may_get_column::<animals::description>(), None);

    builder.try_set_column_ref::<animals::name>("Max")?;

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

    assert_eq!(animal.get_column::<animals::name>().as_str(), "Max");
    assert_eq!(animal.get_column::<animals::description>(), &None);

    // Test with description set to Some value
    let animal_with_desc = animals::table::builder()
        .try_set_column::<animals::name>("Buddy")?
        .try_set_column::<animals::description>(Some("A friendly dog".to_owned()))?
        .insert(&mut conn)?;

    assert_eq!(
        animal_with_desc.description.as_deref(),
        Some("A friendly dog")
    );

    // Test with description explicitly set to None (NULL in database)
    let animal_no_desc = animals::table::builder()
        .try_set_column::<animals::name>("Whiskers")?
        .try_set_column::<animals::description>(None)?
        .insert(&mut conn)?;

    assert_eq!(animal_no_desc.description, None);

    // We attempt to query the inserted animal to ensure everything worked correctly.
    let queried_animal: Animal = animals::table
        .filter(animals::id.eq(animal.id))
        .first(&mut conn)?;
    assert_eq!(animal, queried_animal);

    // We test the chained variant.
    let another_animal = animals::table::builder()
        .try_set_column::<animals::name>("Charlie")?
        .insert(&mut conn)?;

    assert_eq!(
        another_animal.get_column::<animals::name>().as_str(),
        "Charlie"
    );

    assert_ne!(
        animal.get_column::<animals::id>(),
        another_animal.get_column::<animals::id>()
    );

    Ok(())
}

#[test]
fn test_empty_name_rejected() {
    let result = animals::table::builder().try_set_column::<animals::name>(String::new());
    assert_eq!(result.unwrap_err(), NewAnimalError::NameEmpty);
}

#[test]
fn test_whitespace_only_name_rejected() {
    let result = animals::table::builder().try_set_column::<animals::name>("   ".to_string());
    assert_eq!(result.unwrap_err(), NewAnimalError::NameEmpty);
}

#[test]
fn test_name_too_long_rejected() {
    let long_name = "a".repeat(101);
    let result = animals::table::builder().try_set_column::<animals::name>(long_name);
    assert_eq!(result.unwrap_err(), NewAnimalError::NameTooLong);
}

#[test]
fn test_empty_description_rejected() {
    let result =
        animals::table::builder().try_set_column::<animals::description>(Some(String::new()));
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionEmpty);
}

#[test]
fn test_whitespace_only_description_rejected() {
    let result =
        animals::table::builder().try_set_column::<animals::description>(Some("   ".to_string()));
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionEmpty);
}

#[test]
fn test_description_too_long_rejected() {
    let long_desc = "a".repeat(501);
    let result = animals::table::builder().try_set_column::<animals::description>(Some(long_desc));
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionTooLong);
}

#[test]
fn test_none_description_allowed() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = animals::table::builder();
    builder.try_set_column_ref::<animals::description>(None)?;
    assert_eq!(
        builder.may_get_column::<animals::description>(),
        Some(&None)
    );
    Ok(())
}
