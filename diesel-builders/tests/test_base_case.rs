//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod common;
use std::collections::HashMap;

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
        deserialized
            .may_get_column::<animals::name>()
            .map(String::as_str),
        Some("Serialized Animal")
    );
    assert_eq!(
        deserialized.may_get_column::<animals::description>(),
        Some(&Some("Testing serde serialization".to_owned()))
    );

    Ok(())
}

#[test]
fn test_builder_equality() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialEq implementation for TableBuilder
    let builder1 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("A test description".to_owned())?;

    let builder2 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("A test description".to_owned())?;

    let builder3 = animals::table::builder()
        .try_name("Different Animal")?
        .try_description("A test description".to_owned())?;

    let builder4 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("Different description".to_owned())?;

    // Identical builders should be equal
    assert_eq!(builder1, builder2);

    // Different builders should not be equal
    assert_ne!(builder1, builder3);
    assert_ne!(builder1, builder4);
    assert_ne!(builder3, builder4);

    // The builders should also be equal to themselves
    assert_eq!(builder1, builder1);
    assert_eq!(builder2, builder2);
    assert_eq!(builder3, builder3);
    assert_eq!(builder4, builder4);

    Ok(())
}

#[test]
fn test_builder_hash() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Test Hash implementation for TableBuilder
    let builder1 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("A test description".to_owned())?;

    let builder2 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("A test description".to_owned())?;

    let builder3 = animals::table::builder()
        .try_name("Different Animal")?
        .try_description("A test description".to_owned())?;

    let builder4 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("Different description".to_owned())?;

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    builder1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    builder2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    builder3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    builder4.hash(&mut hasher4);
    let hash4 = hasher4.finish();

    // Identical builders should have the same hash
    assert_eq!(hash1, hash2);

    // Different builders should have different hashes
    assert_ne!(hash1, hash3);
    assert_ne!(hash1, hash4);
    assert_ne!(hash3, hash4);

    // Test that builders can be used as HashMap keys
    let mut map = HashMap::new();
    map.insert(builder1.clone(), "value1");
    map.insert(builder3.clone(), "value3");

    assert_eq!(map.get(&builder2), Some(&"value1"));
    assert_eq!(map.get(&builder4), None);
    assert_eq!(map.get(&builder3), Some(&"value3"));

    Ok(())
}

#[test]
fn test_builder_partial_ord() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialOrd implementation for TableBuilder
    let builder1 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("A test description".to_owned())?;

    let builder2 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("A test description".to_owned())?;

    let builder3 = animals::table::builder()
        .try_name("Different Animal")?
        .try_description("A test description".to_owned())?;

    let builder4 = animals::table::builder()
        .try_name("Test Animal")?
        .try_description("Different description".to_owned())?;

    // Identical builders should be equal
    assert_eq!(
        builder1.partial_cmp(&builder2),
        Some(std::cmp::Ordering::Equal)
    );

    // Different builders should have proper ordering
    assert_eq!(
        builder1.partial_cmp(&builder3),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        builder3.partial_cmp(&builder1),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        builder1.partial_cmp(&builder4),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        builder4.partial_cmp(&builder1),
        Some(std::cmp::Ordering::Greater)
    );

    // Test Ord implementation
    assert_eq!(builder1.cmp(&builder2), std::cmp::Ordering::Equal);
    assert_eq!(builder1.cmp(&builder3), std::cmp::Ordering::Greater);
    assert_eq!(builder3.cmp(&builder1), std::cmp::Ordering::Less);
    assert_eq!(builder1.cmp(&builder4), std::cmp::Ordering::Less);
    assert_eq!(builder4.cmp(&builder1), std::cmp::Ordering::Greater);

    Ok(())
}
