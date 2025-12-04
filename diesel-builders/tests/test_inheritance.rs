//! Test case for foreign key based inheritance where Dogs extends
//! Animals. The primary key of Dogs is a foreign key to the primary key
//! of Animals.

mod common;

use common::*;
use diesel::prelude::*;
use diesel_builders::prelude::*;

#[test]
fn test_dog_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create the animals table
    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    // Create the dogs table with foreign key to animals
    diesel::sql_query(CREATE_DOGS_TABLE).execute(&mut conn)?;

    // We create an animal without a dog entry - demonstrating Root derive
    let animal = animals::table::builder()
        .try_name("Generic Animal")?
        .insert(&mut conn)?;

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(animal.id))
        .first(&mut conn)?;
    assert_eq!(loaded_animal, animal);

    // Test TableModel derive - accessing primary key via GetColumn
    assert_eq!(loaded_animal.id(), &animal.id);

    // Now create a dog (which also creates an animal entry via inheritance)
    let dog_builder = dogs::table::builder().try_name("Max")?;

    // Test generated helper traits - fluent API for setting columns
    let dog_builder = dog_builder.breed("Golden Retriever");

    // Test MayGetColumn derive - verifying builder state before insertion
    let breed_value = dog_builder.may_get_column::<dogs::breed>();
    assert_eq!(breed_value, Some(&"Golden Retriever".to_string()));

    let dog = dog_builder.insert(&mut conn)?;

    assert_eq!(dog.breed, "Golden Retriever");

    // Verify the dog can be queried
    let queried_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;
    assert_eq!(dog, queried_dog);

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(dog.id))
        .first(&mut conn)?;

    let loaded_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;

    // Test GetColumn derive on both parent and child models
    assert_eq!(loaded_animal.id(), &dog.id);
    assert_eq!(loaded_animal.name(), "Max");
    assert_eq!(loaded_dog.breed(), "Golden Retriever");
    assert_eq!(loaded_dog, dog);

    Ok(())
}

#[test]
fn test_cat_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create the animals table
    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    // Create the cats table with foreign key to animals
    diesel::sql_query(common::CREATE_CATS_TABLE).execute(&mut conn)?;

    // Now create a cat (which also creates an animal entry via inheritance)
    let cat_builder = common::cats::table::builder()
        .try_name("Whiskers")?
        .try_color("Orange")?;

    // Test MayGetColumn derive on builder to verify state
    let color_value = cat_builder.may_get_column::<common::cats::color>();
    assert_eq!(color_value, Some(&"Orange".to_string()));

    let cat = cat_builder.insert(&mut conn)?;

    assert_eq!(cat.color, "Orange");

    // Verify the cat can be queried
    let queried_cat: common::Cat = common::cats::table
        .filter(common::cats::id.eq(cat.id))
        .first(&mut conn)?;
    assert_eq!(cat, queried_cat);

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(cat.id))
        .first(&mut conn)?;

    let loaded_cat: common::Cat = common::cats::table
        .filter(common::cats::id.eq(cat.id))
        .first(&mut conn)?;

    // Test GetColumn derive - type-safe column access on both models
    assert_eq!(loaded_animal.id(), &cat.id);
    assert_eq!(loaded_animal.name(), "Whiskers");
    assert_eq!(loaded_cat.color(), "Orange");
    assert_eq!(loaded_cat, cat);

    Ok(())
}

#[test]
fn test_dog_insert_fails_when_parent_table_missing() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create only the dogs table, but NOT the animals table (parent)
    diesel::sql_query(CREATE_DOGS_TABLE).execute(&mut conn)?;

    let result = dogs::table::builder()
        .try_name("Max")?
        .breed("Golden Retriever")
        .insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Diesel(_)
    ));

    Ok(())
}

#[test]
fn test_dog_insert_fails_when_child_table_missing() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create only the animals table, but NOT the dogs table (child)
    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    let result = dogs::table::builder()
        .try_name("Max")?
        .breed("Golden Retriever")
        .insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Diesel(_)
    ));

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Dog that extends Animals
    let builder = dogs::table::builder()
        .try_name("Serialized Dog")?
        .breed("German Shepherd");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<dogs::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - breed is the only field directly in NewDog
    assert_eq!(
        deserialized
            .may_get_column::<dogs::breed>()
            .map(String::as_str),
        Some("German Shepherd")
    );

    Ok(())
}
