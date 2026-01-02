//! Test case for foreign key based inheritance where the dependencies
//! form a directed acyclic graph (DAG).
//!
//! We have a root table Animals, which has two descendants Dogs and Cats.
//! Both Dogs and Cats extend Animals via foreign keys. Then, we have a table
//! Pets that extends both Dogs and Cats via foreign keys (diamond pattern).

mod shared;
mod shared_animals;
use shared_animals::*;

use diesel_builders::{BuilderError, prelude::*};

#[test]
fn test_dag() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Insert into animals table
    let animal: Animal = animals::table::builder()
        .try_name("Generic Animal")?
        .insert(&mut conn)?;

    assert_eq!(animal.name(), "Generic Animal");

    // Test GetColumn derive - accessing animal properties type-safely
    assert_eq!(animal.id(), animal.id());
    assert_eq!(animal.name(), animal.name());

    // Insert into dogs table (extends animals)
    // Using helper trait methods for fluent API
    let dog: Dog = dogs::table::builder()
        .try_name("Max the Dog")?
        .breed("Golden Retriever")
        .insert(&mut conn)?;

    assert_eq!(dog.breed(), "Golden Retriever");

    // Test GetColumn derive on descendant table
    assert_eq!(dog.id(), dog.id());
    assert_eq!(dog.breed(), "Golden Retriever");
    let loaded_animal: Animal = dog.ancestor(&mut conn)?;
    assert_eq!(
        loaded_animal.description().as_deref(),
        Some("A generic dog")
    );

    // Insert into cats table (extends animals)
    let cat_builder = cats::table::builder()
        .try_name("Whiskers the Cat")?
        .try_color("Orange")?;

    // Test MayGetColumn derive - checking if optional fields are set
    let color_value = cat_builder.may_get_column_ref::<cats::color>();
    assert!(color_value.is_some());
    assert_eq!(color_value, Some(&"Orange".to_string()));

    let cat: Cat = cat_builder.insert(&mut conn)?;

    assert_eq!(cat.color(), "Orange");

    // Insert into pets table (extends both dogs and cats)
    let pet_builder = pets::table::builder()
        .try_name("Buddy the Pet")?
        .try_breed("Labrador")?
        .try_color("Black")?
        .owner_name("Alice");

    // Test generated helper traits - using fluent API (consumes and returns self)
    let pet_builder = pet_builder.owner_name("Alice Smith"); // Helper method from SetPetOwnerName

    // Test MayGetColumn on builder to verify values before insertion
    let owner_name = pet_builder.may_get_column_ref::<pets::owner_name>();
    assert_eq!(owner_name, Some(&"Alice Smith".to_string()));

    // Test Debug formatting
    let _formatted = format!("{pet_builder:?}");

    let pet: Pet = pet_builder.insert(&mut conn)?;

    assert_eq!(pet.owner_name(), "Alice Smith");

    // Test TableModel derive - using UniquelyIndexedColumn implementations
    assert_eq!(pet.id(), pet.id());

    // Query to verify relationships
    let queried_animal: Animal = pet.ancestor(&mut conn)?;
    assert_eq!(queried_animal.name(), "Buddy the Pet");
    let queried_dog: Dog = pet.ancestor(&mut conn)?;
    assert_eq!(queried_dog.breed(), "Labrador");
    let queried_cat: Cat = pet.ancestor(&mut conn)?;
    assert_eq!(queried_cat.color(), "Black");
    let queried_pet: Pet = Pet::find(pet.id(), &mut conn)?;
    assert_eq!(queried_pet, pet);

    // Test delete cascade with DAG structure
    // Deleting pet should cascade through both dog and cat to animals
    let pet_id = pet.id();
    let deleted_rows = pet.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify pet is deleted
    assert!(!Pet::exists(pet.id(), &mut conn)?);

    // Verify associated dog is deleted
    assert!(!Dog::exists(pet_id, &mut conn)?);

    // Verify associated cat is deleted
    assert!(!Cat::exists(pet_id, &mut conn)?);

    // Verify associated animal is deleted
    assert!(!Animal::exists(pet_id, &mut conn)?);

    // The standalone dog, cat, and animal should still exist
    assert!(Dog::exists(dog.id(), &mut conn)?);

    assert!(Cat::exists(cat.id(), &mut conn)?);

    assert!(Animal::exists(animal.id(), &mut conn)?);

    Ok(())
}

#[test]
fn test_cat_color_empty_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Attempting to create a cat with an empty color should fail validation
    let result = cats::table::builder().try_name("Whiskers")?.try_color("");

    assert_eq!(result.unwrap_err(), NewCatError::ColorEmpty);

    // Also test with whitespace-only color (should also fail)
    let result = cats::table::builder().try_name("Mittens")?.try_color("   ");

    assert_eq!(result.unwrap_err(), NewCatError::ColorEmpty);

    Ok(())
}

#[test]
fn test_diesel_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    // Attempting to create a cat with an empty color should fail validation
    let result = cats::table::builder()
        .try_name("Whiskers")?
        .insert(&mut conn);

    let err = result.unwrap_err();
    assert!(matches!(err, BuilderError::Diesel(_)));

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Pet that extends both Dogs and Cats (DAG structure)
    let builder = pets::table::builder()
        .try_name("Serialized Pet")?
        .breed("Mixed Breed")
        .try_color("Brown")?
        .owner_name("Test Owner");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<pets::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - owner_name is the only field directly in NewPet
    assert_eq!(
        deserialized
            .may_get_column_ref::<pets::owner_name>()
            .map(String::as_str),
        Some("Test Owner")
    );

    Ok(())
}

#[test]
fn test_upsert_dag() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // 1. Upsert on Root (Animal)
    let animal = animals::table::builder()
        .try_name("Original Animal")?
        .insert(&mut conn)?;

    let mut animal_update = animal.clone();
    animal_update.name = "Updated Animal".to_string();

    let updated_animal = animal_update.upsert(&mut conn)?;
    assert_eq!(updated_animal.name, "Updated Animal");

    let queried_animal: Animal = Animal::find(&animal.id, &mut conn)?;
    assert_eq!(queried_animal.name, "Updated Animal");

    // 2. Upsert on Descendant (Dog)
    // Note: upsert currently only updates the specific table columns.
    // If we update a field belonging to Dog (breed), it should work.
    let dog = dogs::table::builder()
        .try_name("Original Dog")?
        .breed("Poodle")
        .insert(&mut conn)?;

    let mut dog_update = dog.clone();
    dog_update.breed = "Standard Poodle".to_string();

    let updated_dog = dog_update.upsert(&mut conn)?;
    assert_eq!(updated_dog.breed, "Standard Poodle");

    let queried_dog: Dog = Dog::find(&dog.id, &mut conn)?;
    assert_eq!(queried_dog.breed, "Standard Poodle");

    // 3. Upsert on Multi-Descendant (Pet)
    // Pet extends Dog and Cat.
    // We update a field belonging to Pet (owner_name).
    let pet = pets::table::builder()
        .try_name("Original Pet")?
        .breed("Mix")
        .try_color("Brown")?
        .owner_name("Original Owner")
        .insert(&mut conn)?;

    let mut pet_update = pet.clone();
    pet_update.owner_name = "Updated Owner".to_string();

    let updated_pet = pet_update.upsert(&mut conn)?;
    assert_eq!(updated_pet.owner_name, "Updated Owner");

    let queried_pet: Pet = Pet::find(&pet.id, &mut conn)?;
    assert_eq!(queried_pet.owner_name, "Updated Owner");

    Ok(())
}
