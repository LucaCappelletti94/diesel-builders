//! Test case for foreign key based inheritance where the dependencies
//! form a chain, with Animals being the root, Dogs extending Animals,
//! and Puppies extending Dogs.

mod common;

use common::*;
use diesel::prelude::*;
use diesel_builders::prelude::*;

#[test]
fn test_inheritance_chain() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create animals table
    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    // Create dogs table (extends animals)
    diesel::sql_query(CREATE_DOGS_TABLE).execute(&mut conn)?;

    // Create puppies table (extends dogs)
    diesel::sql_query(CREATE_PUPPIES_TABLE).execute(&mut conn)?;

    // Insert into animals table
    let animal = animals::table::builder()
        .try_name("Generic Animal")?
        .insert(&mut conn)?;

    assert_eq!(animal.name, "Generic Animal");

    // Insert into dogs table (extends animals)
    let dog = dogs::table::builder()
        .try_name("Max")?
        .breed("Golden Retriever")
        .insert(&mut conn)?;

    assert_eq!(dog.breed, "Golden Retriever");

    // Verify dog can be queried
    let queried_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;
    assert_eq!(queried_dog, dog);

    // Insert into puppies table (extends dogs, transitively extends animals)
    let puppy = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(3)
        .insert(&mut conn)?;

    assert_eq!(puppy.age_months, 3);

    // Verify puppy can be queried
    let queried_puppy: Puppy = puppies::table
        .filter(puppies::id.eq(puppy.id))
        .first(&mut conn)?;
    assert_eq!(queried_puppy, puppy);

    // Verify we can join through the chain: animals -> dogs
    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(dog.id))
        .first(&mut conn)?;
    let loaded_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;

    assert_eq!(loaded_animal.id, loaded_dog.id);
    assert_eq!(loaded_dog, dog);

    // Verify we can join through the chain: dogs -> puppies
    let loaded_dog2: Dog = dogs::table.filter(dogs::id.eq(puppy.id)).first(&mut conn)?;
    let loaded_puppy: Puppy = puppies::table
        .filter(puppies::id.eq(puppy.id))
        .first(&mut conn)?;

    assert_eq!(loaded_dog2.id, loaded_puppy.id);
    assert_eq!(loaded_puppy, puppy);

    // Verify we can join through the full chain: animals -> dogs -> puppies
    let full_chain_animal: Animal = animals::table
        .filter(animals::id.eq(puppy.id))
        .first(&mut conn)?;
    let full_chain_dog: Dog = dogs::table.filter(dogs::id.eq(puppy.id)).first(&mut conn)?;
    let full_chain_puppy: Puppy = puppies::table
        .filter(puppies::id.eq(puppy.id))
        .first(&mut conn)?;

    assert_eq!(full_chain_animal.id, full_chain_dog.id);
    assert_eq!(full_chain_dog.id, full_chain_puppy.id);
    assert_eq!(full_chain_puppy, puppy);

    Ok(())
}
