//! Test case for foreign key based inheritance where the dependencies
//! form a chain, with Animals being the root, Dogs extending Animals,
//! and Puppies extending Dogs.

mod shared;
mod shared_animals;
use diesel_builders::prelude::*;
use shared_animals::*;

#[test]
fn test_inheritance_chain() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    // Insert into animals table
    let builder = animals::table::builder().try_name("Generic Animal")?;
    let builder_clone = builder.clone();
    let animal = builder.insert(&mut conn)?;

    let nested_models = builder_clone.insert_nested(&mut conn)?;
    assert_eq!(nested_models.name(), animal.name());

    assert_eq!(animal.name(), "Generic Animal");

    // Insert into dogs table (extends animals)
    let builder = dogs::table::builder().try_name("Max")?.breed("Golden Retriever");
    let builder_clone = builder.clone();
    let dog = builder.insert(&mut conn)?;

    let nested_models = builder_clone.insert_nested(&mut conn)?;
    assert_eq!(nested_models.breed(), dog.breed());
    assert_eq!(nested_models.name(), "Max");

    assert_eq!(dog.breed(), "Golden Retriever");

    // Verify dog can be queried
    let queried_dog: Dog = Dog::find(dog.get_column_ref::<dogs::id>(), &mut conn)?;
    assert_eq!(queried_dog, dog);

    // Insert into puppies table (extends dogs, transitively extends animals)
    let builder =
        puppies::table::builder().try_name("Buddy")?.breed("Labrador").try_age_months(3)?;

    let builder_clone = builder.clone();
    let puppy = builder.insert(&mut conn)?;

    let nested_models = builder_clone.insert_nested(&mut conn)?;
    assert_eq!(nested_models.age_months(), puppy.age_months());
    assert_eq!(nested_models.breed(), "Labrador");
    assert_eq!(nested_models.name(), "Buddy");

    assert_eq!(*puppy.age_months(), 3);

    // Verify puppy can be queried
    let queried_puppy: Puppy = Puppy::find(puppy.get_column_ref::<puppies::id>(), &mut conn)?;
    assert_eq!(queried_puppy, puppy);

    // Verify we can join through the chain: animals -> dogs
    let loaded_animal: Animal = dog.ancestor(&mut conn)?;
    let loaded_dog: Dog = Dog::find(dog.get_column_ref::<dogs::id>(), &mut conn)?;

    assert_eq!(loaded_animal.get_column::<animals::id>(), loaded_dog.get_column::<dogs::id>());
    assert_eq!(loaded_dog, dog);

    // Verify we can join through the chain: dogs -> puppies
    let loaded_dog2: Dog = puppy.ancestor(&mut conn)?;
    let loaded_puppy: Puppy = Puppy::find(puppy.get_column_ref::<puppies::id>(), &mut conn)?;

    assert_eq!(loaded_dog2.get_column::<dogs::id>(), loaded_puppy.get_column::<puppies::id>());
    assert_eq!(loaded_puppy, puppy);

    // Verify we can join through the full chain: animals -> dogs -> puppies
    let full_chain_animal: Animal = puppy.ancestor(&mut conn)?;
    let full_chain_dog: Dog = puppy.ancestor(&mut conn)?;

    assert_eq!(
        full_chain_animal.get_column::<animals::id>(),
        full_chain_dog.get_column::<dogs::id>()
    );
    assert_eq!(full_chain_dog.get_column::<dogs::id>(), puppy.get_column::<puppies::id>());

    // Test delete cascade through the inheritance chain
    // Deleting puppy should cascade through dogs to animals
    let puppy_id = puppy.get_column_ref::<puppies::id>();
    let deleted_rows = puppy.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify puppy is deleted
    assert!(!Puppy::exists(puppy_id, &mut conn)?);

    // Verify associated dog is deleted
    assert!(!Dog::exists(puppy_id, &mut conn)?);

    // Verify associated animal is deleted
    assert!(!Animal::exists(puppy_id, &mut conn)?);

    // The standalone dog and animal should still exist
    assert!(Dog::exists(dog.get_column_ref::<dogs::id>(), &mut conn)?);

    assert!(Animal::exists(animal.get_column_ref::<animals::id>(), &mut conn)?);

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Puppy that extends Dogs which extends Animals (chain
    // inheritance)
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
    assert_eq!(deserialized.may_get_column_ref::<puppies::age_months>(), Some(&6));

    Ok(())
}
