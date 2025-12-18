//! Test case for foreign key based inheritance where the dependencies
//! form a chain, with Animals being the root, Dogs extending Animals,
//! and Puppies extending Dogs.

mod shared;
mod shared_animals;
use diesel::prelude::*;
use diesel_builders::prelude::*;
use shared_animals::*;

#[test]
fn test_inheritance_chain() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    // Insert into animals table
    let animal = animals::table::builder()
        .try_name("Generic Animal")?
        .insert(&mut conn)?;

    assert_eq!(animal.name(), "Generic Animal");

    // Insert into dogs table (extends animals)
    let dog = dogs::table::builder()
        .try_name("Max")?
        .breed("Golden Retriever")
        .insert(&mut conn)?;

    assert_eq!(dog.breed(), "Golden Retriever");

    // Verify dog can be queried
    let queried_dog: Dog = Dog::find(dog.id(), &mut conn)?;
    assert_eq!(queried_dog, dog);

    // Insert into puppies table (extends dogs, transitively extends animals)
    let puppy = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_age_months(3)?
        .insert(&mut conn)?;

    assert_eq!(*puppy.age_months(), 3);

    // Verify puppy can be queried
    let queried_puppy: Puppy = Puppy::find(puppy.id(), &mut conn)?;
    assert_eq!(queried_puppy, puppy);

    // Verify we can join through the chain: animals -> dogs
    let loaded_animal: Animal = dog.ancestor(&mut conn)?;
    let loaded_dog: Dog = Dog::find(dog.id(), &mut conn)?;

    assert_eq!(loaded_animal.id(), loaded_dog.id());
    assert_eq!(loaded_dog, dog);

    // Verify we can join through the chain: dogs -> puppies
    let loaded_dog2: Dog = puppy.ancestor(&mut conn)?;
    let loaded_puppy: Puppy = Puppy::find(puppy.id(), &mut conn)?;

    assert_eq!(loaded_dog2.id(), loaded_puppy.id());
    assert_eq!(loaded_puppy, puppy);

    // Verify we can join through the full chain: animals -> dogs -> puppies
    let full_chain_animal: Animal = puppy.ancestor(&mut conn)?;
    let full_chain_dog: Dog = puppy.ancestor(&mut conn)?;

    assert_eq!(full_chain_animal.id(), full_chain_dog.id());
    assert_eq!(full_chain_dog.id(), puppy.id());

    // Test delete cascade through the inheritance chain
    // Deleting puppy should cascade through dogs to animals
    let puppy_id = puppy.id();
    let deleted_rows = puppy.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify puppy is deleted
    assert!(!Puppy::exists(puppy_id, &mut conn)?);

    // Verify associated dog is deleted
    assert!(!Dog::exists(puppy_id, &mut conn)?);

    // Verify associated animal is deleted
    assert!(!Animal::exists(puppy_id, &mut conn)?);

    // The standalone dog and animal should still exist
    assert!(Dog::exists(dog.id(), &mut conn)?);

    assert!(Animal::exists(animal.id(), &mut conn)?);

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Puppy that extends Dogs which extends Animals (chain inheritance)
    let builder = puppies::table::builder()
        .try_name("Serialized Puppy")?
        .breed("Beagle")
        .try_age_months(6)?;

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<puppies::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - age_months is the only field directly in NewPuppy
    assert_eq!(
        deserialized.may_get_column_ref::<puppies::age_months>(),
        Some(&6)
    );

    Ok(())
}
