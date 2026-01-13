//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod shared;
mod shared_animals;
use std::{rc::Rc, sync::Arc};

use diesel_builders::prelude::*;
use shared_animals::*;

#[test]
fn test_simple_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    // Test Root derive - animals table is a root with no ancestors
    let mut builder = animals::table::builder();

    // Test MayGetColumn derive - optional fields start as None
    assert_eq!(builder.may_get_column_ref::<animals::name>(), None);
    assert_eq!(builder.may_get_column_ref::<animals::description>(), Some(&None));

    // Test generated TrySetAnimalsName helper trait - fallible setter by reference
    builder.try_name_ref("Max")?;

    // Test MayGetColumn derive - verifying field is set after mutation
    assert_eq!(builder.may_get_column_ref::<animals::name>().map(String::as_str), Some("Max"));
    assert_eq!(builder.may_get_column_ref::<animals::description>(), Some(&None));

    let mut animal = builder.insert(&mut conn)?;

    assert_eq!(animal.name(), "Max");
    assert!(animal.description().is_none());

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

    assert_eq!(animal_with_desc.description().as_deref(), Some("A friendly dog"));

    // Test with description explicitly set to None (NULL in database)
    let animal_no_desc =
        animals::table::builder().try_name("Whiskers")?.try_description(None)?.insert(&mut conn)?;

    assert!(animal_no_desc.description().is_none());

    // We attempt to query the inserted animal to ensure everything worked
    // correctly.
    let queried_animal: Animal = Animal::find(animal.id(), &mut conn)?;
    assert_eq!(animal, queried_animal);

    // Test chained builder pattern with GetColumn derive
    let another_animal = animals::table::builder().try_name("Charlie")?.insert(&mut conn)?;

    // Test GetColumn derive on multiple fields
    assert_eq!(another_animal.name(), "Charlie");

    // Test TableModel derive - verifying unique primary keys
    assert_ne!(animal.id(), another_animal.id());

    // We try to change Animal and use directly Upsert:
    animal.name = "Maximus".to_string();
    let upserted_animal = animal.upsert(&mut conn)?;
    assert_eq!(upserted_animal.name(), "Maximus");
    assert_eq!(upserted_animal.id(), animal.id());
    let reloaded_animal: Animal = Animal::find(animal.id(), &mut conn)?;
    assert_eq!(reloaded_animal, upserted_animal);

    // We try to delete the first animal using the ModelDelete trait
    let deleted_rows = animal.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // We check that the animal is indeed deleted
    let remaining_animals: Vec<Animal> = animals::table.load(&mut conn)?;
    assert!(!remaining_animals.contains(&animal));
    assert!(remaining_animals.contains(&another_animal));

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
    let result = animals::table::builder().try_description(String::new());
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionEmpty);
}

#[test]
fn test_whitespace_only_description_rejected() {
    let result = animals::table::builder().try_description("   ".to_string());
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionEmpty);
}

#[test]
fn test_description_too_long_rejected() {
    let long_desc = "a".repeat(501);
    let result = animals::table::builder().try_description(long_desc);
    assert_eq!(result.unwrap_err(), NewAnimalError::DescriptionTooLong);
}

#[test]
fn test_none_description_allowed() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = animals::table::builder();
    builder.try_description_ref(None)?;
    assert_eq!(builder.may_get_column_ref::<animals::description>(), Some(&None));
    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder with some values set
    let builder = animals::table::builder()
        .try_name("Serialized Animal")?
        .try_description("Testing serde serialization".to_owned())?;

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<animals::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(
        deserialized.may_get_column_ref::<animals::name>().map(String::as_str),
        Some("Serialized Animal")
    );
    assert_eq!(
        deserialized.may_get_column_ref::<animals::description>(),
        Some(&Some("Testing serde serialization".to_owned()))
    );

    Ok(())
}

#[test]
fn completed_table_builder_bundle_has_table() {
    use diesel::associations::HasTable;
    let _table: animals::table =
        <diesel_builders::CompletedTableBuilderBundle<animals::table> as HasTable>::table();
    let _table: animals::table = <diesel_builders::table_builder::RecursiveTableBuilder<
        animals::table,
        typenum::U0,
        (),
    > as HasTable>::table();
}

#[test]
fn test_get_column_blanket_impls() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    let mut builder = animals::table::builder();
    builder.try_name_ref("Test Animal")?;
    builder.try_description_ref(Some("A test description".to_string()))?;
    let animal = builder.insert(&mut conn)?;

    // Test reference blanket impl
    let animal_ref = &animal;
    assert_eq!(animal_ref.get_column::<animals::name>(), "Test Animal");
    assert_eq!(
        animal_ref.get_column::<animals::description>(),
        Some("A test description".to_string())
    );
    assert_eq!(animal_ref.get_column_ref::<animals::name>(), "Test Animal");
    assert_eq!(
        animal_ref.get_column_ref::<animals::description>(),
        &Some("A test description".to_string())
    );

    // Test Box blanket impl
    let animal_box = Box::new(animal.clone());
    assert_eq!(animal_box.get_column::<animals::name>(), "Test Animal");
    assert_eq!(
        animal_box.get_column::<animals::description>(),
        Some("A test description".to_string())
    );
    assert_eq!(animal_box.get_column_ref::<animals::name>(), "Test Animal");
    assert_eq!(
        animal_box.get_column_ref::<animals::description>(),
        &Some("A test description".to_string())
    );

    // Test Rc blanket impl
    let animal_rc = Rc::new(animal.clone());
    assert_eq!(animal_rc.get_column::<animals::name>(), "Test Animal");
    assert_eq!(
        animal_rc.get_column::<animals::description>(),
        Some("A test description".to_string())
    );
    assert_eq!(animal_rc.get_column_ref::<animals::name>(), "Test Animal");
    assert_eq!(
        animal_rc.get_column_ref::<animals::description>(),
        &Some("A test description".to_string())
    );

    // Test Arc blanket impl
    let arc_animal = Arc::new(animal.clone());
    assert_eq!(arc_animal.get_column::<animals::name>(), "Test Animal");
    assert_eq!(
        arc_animal.get_column::<animals::description>(),
        Some("A test description".to_string())
    );
    assert_eq!(arc_animal.get_column_ref::<animals::name>(), "Test Animal");
    assert_eq!(
        arc_animal.get_column_ref::<animals::description>(),
        &Some("A test description".to_string())
    );

    Ok(())
}
