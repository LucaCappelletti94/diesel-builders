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

    let dyn_name = animals::name.into();
    let name_val = nested_models.try_get_dynamic_column_ref::<String>(dyn_name)?;
    assert_eq!(name_val, Some(animal.name()));

    // Test retrieval of optional column that is currently None
    let dyn_desc = animals::description.into();
    let desc_val = nested_models.try_get_dynamic_column_ref::<String>(dyn_desc)?;
    assert_eq!(desc_val, None);

    assert_eq!(animal.name(), "Generic Animal");

    // Insert into dogs table (extends animals)
    let builder = dogs::table::builder().try_name("Max")?.breed("Golden Retriever");
    let builder_clone = builder.clone();
    let dog = builder.insert(&mut conn)?;

    let nested_models = builder_clone.insert_nested(&mut conn)?;
    assert_eq!(nested_models.breed(), dog.breed());
    assert_eq!(nested_models.name(), "Max");

    let dyn_breed = dogs::breed.into();
    let breed_val = nested_models.try_get_dynamic_column_ref::<String>(dyn_breed)?;
    assert_eq!(breed_val, Some(dog.breed()));

    let name_val = nested_models.try_get_dynamic_column_ref::<String>(dyn_name)?;
    assert_eq!(name_val, Some(&"Max".to_string()));

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

    let dyn_age = puppies::age_months.into();
    let age_val = nested_models.try_get_dynamic_column_ref::<i32>(dyn_age)?;
    assert_eq!(age_val, Some(puppy.age_months()));

    // Test owned get
    let breed_val_owned = nested_models.try_get_dynamic_column::<String>(dyn_breed)?;
    assert_eq!(breed_val_owned, Some("Labrador".to_string()));

    let name_val_owned = nested_models.try_get_dynamic_column::<String>(dyn_name)?;
    assert_eq!(name_val_owned, Some("Buddy".to_string()));

    // Test unknown column
    let dyn_cat_color = cats::color.into();
    let result = nested_models.try_get_dynamic_column_ref::<String>(dyn_cat_color);
    assert!(matches!(
        result,
        Err(diesel_builders::builder_error::DynamicColumnError::UnknownColumn { .. })
    ));

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

#[test]
fn test_load_nested_traits_chain() -> Result<(), Box<dyn std::error::Error>> {
    use diesel_builders::load_nested_query_builder::{LoadNestedFirst, LoadNestedMany};

    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    // 1. Insert Animal -> Dog -> Puppy 1
    let puppy1 = puppies::table::builder()
        .try_name("Puppy1")?
        .breed("BreedA")
        .try_age_months(1)?
        .insert_nested(&mut conn)?;

    // 2. Insert Animal -> Dog -> Puppy 2 (Same Breed)
    let puppy2 = puppies::table::builder()
        .try_name("Puppy2")?
        .breed("BreedA")
        .try_age_months(2)?
        .insert_nested(&mut conn)?;

    // 3. Insert Animal -> Dog -> Puppy 3 (Different Breed)
    let puppy3 = puppies::table::builder()
        .try_name("Puppy3")?
        .breed("BreedB")
        .try_age_months(3)?
        .insert_nested(&mut conn)?;

    // Test LoadNestedMany filtering by dogs::breed ("BreedA")
    // This verifies we can filter on an ancestor column (dogs) while querying leaf
    // (puppies) and retrieve the full nested structure.
    let loaded_a = <(dogs::breed,) as LoadNestedMany<puppies::table, _>>::load_nested_many(
        ("BreedA",),
        &mut conn,
    )?;

    assert_eq!(loaded_a.len(), 2);
    assert!(loaded_a.contains(&puppy1), "Loaded puppies should contain Puppy1");
    assert!(loaded_a.contains(&puppy2), "Loaded puppies should contain Puppy2");

    // Test LoadNestedFirst for Puppy3 (BreedB)
    let loaded_b = <(dogs::breed,) as LoadNestedFirst<puppies::table, _>>::load_nested_first(
        ("BreedB",),
        &mut conn,
    )?;

    assert_eq!(loaded_b, puppy3, "Loaded puppy should match Puppy3");

    Ok(())
}
