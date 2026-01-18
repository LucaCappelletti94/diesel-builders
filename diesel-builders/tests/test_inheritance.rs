//! Test case for foreign key based inheritance where Dogs extends
//! Animals. The primary key of Dogs is a foreign key to the primary key
//! of Animals.

mod shared;
mod shared_animals;
use diesel::prelude::*;
use diesel_builders::prelude::*;
use shared_animals::*;

#[test]
fn test_dog_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // We create an animal without a dog entry - demonstrating Root derive
    let animal = animals::table::builder().try_name("Generic Animal")?.insert(&mut conn)?;

    let loaded_animal: Animal = Animal::find(animal.id(), &mut conn)?;
    assert_eq!(loaded_animal, animal.clone());

    // Test TableModel derive - accessing primary key via GetColumn
    assert_eq!(loaded_animal.id(), animal.id());

    // Now create a dog (which also creates an animal entry via inheritance)
    let dog_builder = dogs::table::builder().try_name("Max")?;

    // Test generated helper traits - fluent API for setting columns
    let dog_builder = dog_builder.breed("Golden Retriever");

    // Test MayGetColumn derive - verifying builder state before insertion
    let breed_value = dog_builder.may_get_column_ref::<dogs::breed>();
    assert_eq!(breed_value, Some(&"Golden Retriever".to_string()));

    let dog = dog_builder.insert(&mut conn)?;

    assert_eq!(dog.breed(), "Golden Retriever");

    // Verify the dog can be queried
    let queried_dog: Dog = Dog::find(dog.id(), &mut conn)?;
    assert_eq!(dog, queried_dog);

    let loaded_animal: Animal = dog.ancestor(&mut conn)?;

    let loaded_dog: Dog = Dog::find(dog.id(), &mut conn)?;

    // Test GetColumn derive on both parent and child models
    assert_eq!(loaded_animal.id(), dog.id());
    assert_eq!(loaded_animal.name(), "Max");
    assert_eq!(loaded_animal.description().as_deref(), Some("A generic dog"));
    assert_eq!(loaded_dog.breed(), "Golden Retriever");
    assert_eq!(loaded_dog, dog);

    // Test delete cascade - deleting dog should cascade delete from animals table
    let dog_id = dog.id();
    let deleted_rows = dog.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify the dog is deleted
    assert!(!Dog::exists(dog_id, &mut conn)?);

    // Verify the associated animal is also deleted due to CASCADE
    assert!(!Animal::exists(dog_id, &mut conn)?);

    // The standalone animal should still exist
    assert!(Animal::exists(animal.id(), &mut conn)?);

    Ok(())
}

#[test]
fn test_cat_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    // Now create a cat (which also creates an animal entry via inheritance)
    let cat_builder = cats::table::builder().try_name("Whiskers")?.try_color("Orange")?;

    // Test MayGetColumn derive on builder to verify state
    let color_value = cat_builder.may_get_column_ref::<cats::color>();
    assert_eq!(color_value, Some(&"Orange".to_string()));

    let cat = cat_builder.insert(&mut conn)?;

    assert_eq!(cat.color(), "Orange");

    // Verify the cat can be queried
    let queried_cat: Cat = Cat::find(cat.id(), &mut conn)?;
    assert_eq!(cat, queried_cat);

    let loaded_animal: Animal = cat.ancestor(&mut conn)?;

    let loaded_cat: Cat = Cat::find(cat.id(), &mut conn)?;

    // Test GetColumn derive - type-safe column access on both models
    assert_eq!(loaded_animal.id(), cat.id());
    assert_eq!(loaded_animal.name(), "Whiskers");
    assert_eq!(loaded_cat.color(), "Orange");
    assert_eq!(loaded_cat, cat);

    // Test delete cascade for cat
    let cat_id = cat.id();
    let deleted_rows = cat.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify the cat is deleted
    assert!(!Cat::exists(cat_id, &mut conn)?);

    // Verify the associated animal is also deleted due to CASCADE
    assert!(!Animal::exists(cat_id, &mut conn)?);

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Dog that extends Animals
    let builder = dogs::table::builder().try_name("Serialized Dog")?.breed("German Shepherd");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<dogs::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - breed is the only field directly in NewDog
    assert_eq!(
        deserialized.may_get_column_ref::<dogs::breed>().map(String::as_str),
        Some("German Shepherd")
    );

    Ok(())
}

#[test]
fn test_dynamic_column_setting_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Create a dog using dynamic column setting
    let dyn_breed_column = dogs::breed.into();
    let dyn_name_column = animals::name.into();

    let dog = dogs::table::builder()
        .try_set_dynamic_column(dyn_name_column, &"Dynamic Dog".to_owned())?
        .try_set_dynamic_column(dyn_breed_column, &"Dynamic Breed".to_owned())?
        .insert(&mut conn)?;

    // Load the ancestor to check name
    let loaded_animal: Animal = dog.ancestor(&mut conn)?;
    assert_eq!(loaded_animal.name(), "Dynamic Dog");
    assert_eq!(dog.breed(), "Dynamic Breed");

    // Verify via query
    let queried_dog: Dog = Dog::find(dog.id(), &mut conn)?;
    let queried_animal: Animal = queried_dog.ancestor(&mut conn)?;
    assert_eq!(queried_animal.name(), "Dynamic Dog");
    assert_eq!(queried_dog.breed(), "Dynamic Breed");

    Ok(())
}
