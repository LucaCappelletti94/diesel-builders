//! Test case for foreign key based inheritance where the dependencies
//! form a directed acyclic graph (DAG).
//!
//! We have a root table Animals, which has two descendants Dogs and Cats.
//! Both Dogs and Cats extend Animals via foreign keys. Then, we have a table
//! Pets that extends both Dogs and Cats via foreign keys (diamond pattern).

mod common;

use common::*;
use diesel::prelude::*;
use diesel_builders::prelude::*;

#[test]
fn test_dag() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create animals table
    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    // Create dogs table (extends animals)
    diesel::sql_query(CREATE_DOGS_TABLE).execute(&mut conn)?;

    // Create cats table (extends animals)
    diesel::sql_query(CREATE_CATS_TABLE).execute(&mut conn)?;

    // Create pets table (extends both dogs and cats)
    diesel::sql_query(CREATE_PETS_TABLE).execute(&mut conn)?;

    // Insert into animals table
    let animal: Animal = animals::table::builder()
        .try_name("Generic Animal")?
        .insert(&mut conn)?;

    assert_eq!(animal.name, "Generic Animal");

    // Test GetColumn derive - accessing animal properties type-safely
    assert_eq!(animal.id(), &animal.id);
    assert_eq!(animal.name(), &animal.name);

    // Insert into dogs table (extends animals)
    // Using helper trait methods for fluent API
    let dog: Dog = dogs::table::builder()
        .try_name("Max the Dog")?
        .breed("Golden Retriever")
        .insert(&mut conn)?;

    assert_eq!(dog.breed, "Golden Retriever");

    // Test GetColumn derive on descendant table
    assert_eq!(dog.id(), &dog.id);
    assert_eq!(dog.breed(), "Golden Retriever");

    // Insert into cats table (extends animals)
    let cat_builder = cats::table::builder()
        .try_name("Whiskers the Cat")?
        .try_color("Orange")?;

    // Test MayGetColumn derive - checking if optional fields are set
    let color_value = cat_builder.may_get_column::<cats::color>();
    assert!(color_value.is_some());
    assert_eq!(color_value, Some(&"Orange".to_string()));

    let cat: Cat = cat_builder.insert(&mut conn)?;

    assert_eq!(cat.color, "Orange");

    // Insert into pets table (extends both dogs and cats)
    let pet_builder = pets::table::builder()
        .try_name("Buddy the Pet")?
        .breed("Labrador")
        .try_color("Black")?
        .owner_name("Alice");

    // Test generated helper traits - using fluent API (consumes and returns self)
    let pet_builder = pet_builder.owner_name("Alice Smith"); // Helper method from SetPetOwnerName

    // Test MayGetColumn on builder to verify values before insertion
    let owner_name = pet_builder.may_get_column::<pets::owner_name>();
    assert_eq!(owner_name, Some(&"Alice Smith".to_string()));

    // Test Debug formatting
    let _formatted = format!("{pet_builder:?}");

    let pet: Pet = pet_builder.insert(&mut conn)?;

    assert_eq!(pet.owner_name, "Alice Smith");

    // Test TableModel derive - using IndexedColumn implementations
    assert_eq!(pet.id(), &pet.id);

    // Query to verify relationships
    let queried_animal: Animal = animals::table
        .filter(animals::id.eq(pet.id))
        .first(&mut conn)?;
    assert_eq!(queried_animal.name, "Buddy the Pet");
    let queried_dog: Dog = dogs::table.filter(dogs::id.eq(pet.id)).first(&mut conn)?;
    assert_eq!(queried_dog.breed, "Labrador");
    let queried_cat: Cat = cats::table.filter(cats::id.eq(pet.id)).first(&mut conn)?;
    assert_eq!(queried_cat.color, "Black");
    let queried_pet: Pet = pets::table.filter(pets::id.eq(pet.id)).first(&mut conn)?;
    assert_eq!(queried_pet, pet);

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
            .may_get_column::<pets::owner_name>()
            .map(String::as_str),
        Some("Test Owner")
    );

    Ok(())
}
