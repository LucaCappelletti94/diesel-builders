//! Test case for foreign key based inheritance where the dependencies
//! form a directed acyclic graph (DAG).
//!
//! We have a root table Animals, which has two descendants Dogs and Cats.
//! Both Dogs and Cats extend Animals via foreign keys. Then, we have a table
//! Pets that extends both Dogs and Cats via foreign keys (diamond pattern).

mod common;

use common::{
    Animal, CREATE_ANIMALS_TABLE, CREATE_CATS_TABLE, CREATE_DOGS_TABLE, CREATE_PETS_TABLE, Cat,
    Dog, Pet, animals, cats, dogs, pets,
};
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
        .try_set_column::<animals::name>("Generic Animal")?
        .insert(&mut conn)?;

    assert_eq!(animal.name, "Generic Animal");

    // Insert into dogs table (extends animals)
    let dog: Dog = dogs::table::builder()
        .try_set_column::<animals::name>("Max the Dog")?
        .set_column::<dogs::breed>("Golden Retriever")
        .insert(&mut conn)?;

    assert_eq!(dog.breed, "Golden Retriever");

    // Insert into cats table (extends animals)
    let cat: Cat = cats::table::builder()
        .try_set_column::<animals::name>("Whiskers the Cat")?
        .try_set_column::<cats::color>("Orange")?
        .insert(&mut conn)?;

    assert_eq!(cat.color, "Orange");

    // Insert into pets table (extends both dogs and cats)
    let pet_builder = pets::table::builder()
        .try_set_column::<animals::name>("Buddy the Pet")?
        .set_column::<dogs::breed>("Labrador")
        .try_set_column::<cats::color>("Black")?
        .set_column::<pets::owner_name>("Alice");

    // Test Debug formatting
    let _formatted = format!("{pet_builder:?}");

    let pet: Pet = pet_builder.insert(&mut conn)?;

    assert_eq!(pet.owner_name, "Alice");

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
