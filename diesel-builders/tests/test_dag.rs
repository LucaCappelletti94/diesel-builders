//! Test case for foreign key based inheritance where the dependencies
//! form a directed acyclic graph (DAG).
//!
//! We have a root table Animals, which has two descendants Dogs and Cats.
//! Both Dogs and Cats extend Animals via foreign keys. Then, we have a table
//! Pets that extends both Dogs and Cats via foreign keys (diamond pattern).

mod shared;
mod shared_animals;
use diesel_builders::{
    BuilderError, NestedTables, load_nested_query_builder::LoadNestedFirst, prelude::*,
};
use shared_animals::*;

#[test]
fn test_dag() -> Result<(), Box<dyn std::error::Error>> {
    type PetNestedModels = <<pets::table as diesel_builders::DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels;

    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Insert into animals table
    let animal: Animal = animals::table::builder().try_name("Generic Animal")?.insert(&mut conn)?;

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
    assert_eq!(loaded_animal.description().as_deref(), Some("A generic dog"));

    // Insert into cats table (extends animals)
    let cat_builder = cats::table::builder().try_name("Whiskers the Cat")?.try_color("Orange")?;

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

    let builder_clone = pet_builder.clone();

    // Test MayGetColumn on builder to verify values before insertion
    let owner_name = pet_builder.may_get_column_ref::<pets::owner_name>();
    assert_eq!(owner_name, Some(&"Alice Smith".to_string()));

    // Test Debug formatting
    let _formatted = format!("{pet_builder:?}");

    let pet: Pet = pet_builder.insert(&mut conn)?;

    let nested_models = builder_clone.insert_nested(&mut conn)?;

    // Test dynamic column retrieval on nested results
    let dyn_owner = pets::owner_name.into();
    let owner_val = nested_models.try_get_dynamic_column_ref(dyn_owner)?;
    assert_eq!(owner_val, Some(pet.owner_name()));

    let dyn_breed = dogs::breed.into();
    let breed_val = nested_models.try_get_dynamic_column_ref(dyn_breed)?;
    assert_eq!(breed_val, Some(&"Labrador".to_string()));

    let dyn_color = cats::color.into();
    let color_val = nested_models.try_get_dynamic_column_ref(dyn_color)?;
    assert_eq!(color_val, Some(&"Black".to_string()));

    let dyn_name = animals::name.into();
    let name_val = nested_models.try_get_dynamic_column_ref(dyn_name)?;
    assert_eq!(name_val, Some(&"Buddy the Pet".to_string()));

    // Test owned get
    let breed_val_owned = nested_models.try_get_dynamic_column(dyn_breed)?;
    assert_eq!(breed_val_owned, Some("Labrador".to_string()));

    let color_val_owned = nested_models.try_get_dynamic_column(dyn_color)?;
    assert_eq!(color_val_owned, Some("Black".to_string()));

    assert_eq!(pet.owner_name(), "Alice Smith");

    // Test dynamic foreign key iteration
    let animal_fk_refs: Vec<_> = nested_models
        .iter_dynamic_match_simple((animals::id.into(),))
        .collect::<Result<Vec<_>, _>>()?;

    let animal_fk_match_refs: Vec<_> = nested_models
        .iter_dynamic_match_full((animals::id.into(),))
        .collect::<Result<Vec<_>, _>>()?;

    // We expect 3 references:
    // 1. Dog -> Animal
    // 2. Cat -> Animal
    // 3. Pet -> Animal (since Pet declares 'animals' in ancestors, acts as logical
    //    FK)
    assert_eq!(animal_fk_refs.len(), 3);
    assert_eq!(animal_fk_match_refs.len(), 3);

    // All FKs should point to the same animal ID (inheritance)
    let first_fk = &animal_fk_refs[0];
    assert_ne!(first_fk.0, Some(pet.id()));

    for fk in &animal_fk_refs {
        assert_eq!(fk, first_fk);
    }
    let first_fk = &animal_fk_match_refs[0];
    assert_ne!(first_fk.0, pet.id());

    for fk in &animal_fk_match_refs {
        assert_eq!(fk, first_fk);
    }

    // Test dynamic foreign key columns iteration
    let dynamic_fk_cols: Vec<_> =
        PetNestedModels::iter_dynamic_foreign_key_columns((animals::id.into(),))
            .collect::<Vec<_>>();

    assert_eq!(dynamic_fk_cols.len(), 3);

    // Verify expected columns (pets::id, dogs::id, cats::id)
    // Note: Implicit casting to DynColumn<i32> via From
    let dyn_dogs_id: diesel_builders::DynColumn<i32> = dogs::id.into();
    let dyn_cats_id: diesel_builders::DynColumn<i32> = cats::id.into();
    let dyn_pets_id: diesel_builders::DynColumn<i32> = pets::id.into();

    // Check presence of each expected FK column
    assert_eq!(dynamic_fk_cols, vec![(dyn_dogs_id,), (dyn_cats_id,), (dyn_pets_id,),]);

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
    let result = cats::table::builder().try_name("Whiskers")?.insert(&mut conn);

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
        deserialized.may_get_column_ref::<pets::owner_name>().map(String::as_str),
        Some("Test Owner")
    );

    Ok(())
}

#[test]
fn test_upsert_dag() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // 1. Upsert on Root (Animal)
    let animal = animals::table::builder().try_name("Original Animal")?.insert(&mut conn)?;

    let mut animal_update = animal.clone();
    animal_update.set_name("Updated Animal".to_string());

    let updated_animal = animal_update.upsert(&mut conn)?;
    assert_eq!(updated_animal.get_column::<animals::name>(), "Updated Animal");

    let queried_animal: Animal = Animal::find(animal.get_column_ref::<animals::id>(), &mut conn)?;
    assert_eq!(queried_animal.get_column::<animals::name>(), "Updated Animal");

    // 2. Upsert on Descendant (Dog)
    // Note: upsert currently only updates the specific table columns.
    // If we update a field belonging to Dog (breed), it should work.
    let dog = dogs::table::builder().try_name("Original Dog")?.breed("Poodle").insert(&mut conn)?;

    let mut dog_update = dog.clone();
    dog_update.set_breed("Standard Poodle".to_string());

    let updated_dog = dog_update.upsert(&mut conn)?;
    assert_eq!(updated_dog.get_column::<dogs::breed>(), "Standard Poodle");

    let queried_dog: Dog = Dog::find(dog.get_column_ref::<dogs::id>(), &mut conn)?;
    assert_eq!(queried_dog.get_column::<dogs::breed>(), "Standard Poodle");

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
    pet_update.set_owner_name("Updated Owner".to_string());

    let updated_pet = pet_update.upsert(&mut conn)?;
    assert_eq!(updated_pet.get_column::<pets::owner_name>(), "Updated Owner");

    let queried_pet: Pet = Pet::find(pet.get_column_ref::<pets::id>(), &mut conn)?;
    assert_eq!(queried_pet.get_column::<pets::owner_name>(), "Updated Owner");

    Ok(())
}

allow_tables_to_appear_in_same_query!(dogs, cats);

#[test]
fn test_load_nested_traits_dag() -> Result<(), Box<dyn std::error::Error>> {
    use diesel_builders::load_nested_query_builder::LoadNestedFirst;

    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Insert Pet
    let nested_pet = pets::table::builder()
        .try_name("Diamond Pet")?
        .breed("Multibreed")
        .try_color("Rainbow")?
        .owner_name("Owner")
        .insert_nested(&mut conn)?;

    let loaded = <(pets::owner_name,) as LoadNestedFirst<pets::table, _>>::load_nested_first(
        ("Owner",),
        &mut conn,
    )?;

    assert_eq!(loaded, nested_pet);

    Ok(())
}

#[test]
fn test_load_nested_many_variants_dag() -> Result<(), Box<dyn std::error::Error>> {
    use diesel_builders::load_nested_query_builder::{
        LoadNestedMany, LoadNestedPaginated, LoadNestedSorted,
    };

    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // 1. Insert Pet 1: Alice (Breed A, Color Red)
    let pet1 = pets::table::builder()
        .try_name("Alice")?
        .breed("BreedA")
        .try_color("Red")?
        .owner_name("Owner1")
        .insert_nested(&mut conn)?;

    // 2. Insert Pet 2: Bob (Breed A, Color Blue)
    let pet2 = pets::table::builder()
        .try_name("Bob")?
        .breed("BreedA")
        .try_color("Blue")?
        .owner_name("Owner2")
        .insert_nested(&mut conn)?;

    // 3. Insert Pet 3: Charlie (Breed B, Color Green)
    let pet3 = pets::table::builder()
        .try_name("Charlie")?
        .breed("BreedB")
        .try_color("Green")?
        .owner_name("Owner3")
        .insert_nested(&mut conn)?;

    // --- Test LoadNestedMany ---
    // Filter by Breed A. Should match Alice and Bob.
    let many_res = <(dogs::breed,) as LoadNestedMany<pets::table, _>>::load_nested_many(
        ("BreedA",),
        &mut conn,
    )?;

    assert_eq!(many_res.len(), 2);
    assert!(many_res.contains(&pet1), "Loaded pets should contain Alice");
    assert!(many_res.contains(&pet2), "Loaded pets should contain Bob");

    // We load the first two pets inserted with Breed B
    let res_b = <(dogs::breed,) as LoadNestedFirst<pets::table, _>>::load_nested_first(
        ("BreedB",),
        &mut conn,
    )?;
    assert_eq!(res_b, pet3, "Loaded pet should be Charlie");

    // --- Test LoadNestedSorted ---
    // Filter by Breed A. Default sort is by PK (ASC usually).
    // Alice (id 1 typically) should come before Bob (id 2).
    let sorted_res = <(dogs::breed,) as LoadNestedSorted<pets::table, _>>::load_nested_sorted(
        ("BreedA",),
        &mut conn,
    )?;

    assert_eq!(sorted_res.len(), 2);
    assert_eq!(sorted_res[0], pet1, "First sorted pet should be Alice");
    assert_eq!(sorted_res[1], pet2, "Second sorted pet should be Bob");

    // --- Test LoadNestedPaginated ---
    // Filter by Breed A. Sort Default (PK ASC).
    // Offset 0, Limit 1 -> Alice
    let paginated_res =
        <(dogs::breed,) as LoadNestedPaginated<pets::table, _>>::load_nested_paginated(
            ("BreedA",),
            0, // offset
            1, // limit
            &mut conn,
        )?;

    assert_eq!(paginated_res[0], pet1, "Paginated first pet should be Alice");

    // Offset 1, Limit 1 -> Bob
    let paginated_res_2 =
        <(dogs::breed,) as LoadNestedPaginated<pets::table, _>>::load_nested_paginated(
            ("BreedA",),
            1, // offset
            1, // limit
            &mut conn,
        )?;

    assert_eq!(paginated_res_2[0], pet2, "Paginated second pet should be Bob");

    Ok(())
}
