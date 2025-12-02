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

    // We create an animal without a dog entry
    let animal = animals::table::builder()
        .try_set_column::<animals::name>("Generic Animal")?
        .insert(&mut conn)?;

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(animal.id))
        .first(&mut conn)?;
    assert_eq!(loaded_animal, animal);

    // Now create a dog (which also creates an animal entry)
    let dog = dogs::table::builder()
        .try_set_column::<animals::name>("Max")?
        .set_column::<dogs::breed>("Golden Retriever")
        .insert(&mut conn)?;

    assert_eq!(dog.breed, "Golden Retriever");

    // Verify the dog can be queried
    let queried_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;
    assert_eq!(dog, queried_dog);

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(dog.id))
        .first(&mut conn)?;

    let loaded_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;

    assert_eq!(loaded_animal.get_column::<animals::id>(), &dog.id);
    assert_eq!(loaded_animal.get_column::<animals::name>(), "Max");
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

    // Now create a cat (which also creates an animal entry)
    let cat = common::cats::table::builder()
        .try_set_column::<common::animals::name>("Whiskers")?
        .try_set_column::<common::cats::color>("Orange")?
        .insert(&mut conn)?;

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

    assert_eq!(loaded_animal.get_column::<animals::id>(), &cat.id);
    assert_eq!(loaded_animal.get_column::<animals::name>(), "Whiskers");
    assert_eq!(loaded_cat, cat);

    Ok(())
}
