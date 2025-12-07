//! Test case for foreign key based inheritance where Dogs extends
//! Animals. The primary key of Dogs is a foreign key to the primary key
//! of Animals.

mod common;
use std::collections::HashMap;

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

    // We create an animal without a dog entry - demonstrating Root derive
    let animal = animals::table::builder()
        .try_name("Generic Animal")?
        .insert(&mut conn)?;

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(animal.id))
        .first(&mut conn)?;
    assert_eq!(loaded_animal, animal);

    // Test TableModel derive - accessing primary key via GetColumn
    assert_eq!(loaded_animal.id(), &animal.id);

    // Now create a dog (which also creates an animal entry via inheritance)
    let dog_builder = dogs::table::builder().try_name("Max")?;

    // Test generated helper traits - fluent API for setting columns
    let dog_builder = dog_builder.breed("Golden Retriever");

    // Test MayGetColumn derive - verifying builder state before insertion
    let breed_value = dog_builder.may_get_column_ref::<dogs::breed>();
    assert_eq!(breed_value, Some(&"Golden Retriever".to_string()));

    let dog = dog_builder.insert(&mut conn)?;

    assert_eq!(dog.breed, "Golden Retriever");

    // Verify the dog can be queried
    let queried_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;
    assert_eq!(dog, queried_dog);

    let loaded_animal: Animal = animals::table
        .filter(animals::id.eq(dog.id))
        .first(&mut conn)?;

    let loaded_dog: Dog = dogs::table.filter(dogs::id.eq(dog.id)).first(&mut conn)?;

    // Test GetColumn derive on both parent and child models
    assert_eq!(loaded_animal.id(), &dog.id);
    assert_eq!(loaded_animal.name(), "Max");
    assert_eq!(loaded_dog.breed(), "Golden Retriever");
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

    // Now create a cat (which also creates an animal entry via inheritance)
    let cat_builder = common::cats::table::builder()
        .try_name("Whiskers")?
        .try_color("Orange")?;

    // Test MayGetColumn derive on builder to verify state
    let color_value = cat_builder.may_get_column_ref::<common::cats::color>();
    assert_eq!(color_value, Some(&"Orange".to_string()));

    let cat = cat_builder.insert(&mut conn)?;

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

    // Test GetColumn derive - type-safe column access on both models
    assert_eq!(loaded_animal.id(), &cat.id);
    assert_eq!(loaded_animal.name(), "Whiskers");
    assert_eq!(loaded_cat.color(), "Orange");
    assert_eq!(loaded_cat, cat);

    Ok(())
}

#[test]
fn test_dog_insert_fails_when_parent_table_missing() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create only the dogs table, but NOT the animals table (parent)
    diesel::sql_query(CREATE_DOGS_TABLE).execute(&mut conn)?;

    let result = dogs::table::builder()
        .try_name("Max")?
        .breed("Golden Retriever")
        .insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Diesel(_)
    ));

    Ok(())
}

#[test]
fn test_dog_insert_fails_when_child_table_missing() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    // Create only the animals table, but NOT the dogs table (child)
    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(&mut conn)?;

    let result = dogs::table::builder()
        .try_name("Max")?
        .breed("Golden Retriever")
        .insert(&mut conn);

    assert!(matches!(
        result.unwrap_err(),
        diesel_builders::BuilderError::Diesel(_)
    ));

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Dog that extends Animals
    let builder = dogs::table::builder()
        .try_name("Serialized Dog")?
        .breed("German Shepherd");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<dogs::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - breed is the only field directly in NewDog
    assert_eq!(
        deserialized
            .may_get_column_ref::<dogs::breed>()
            .map(String::as_str),
        Some("German Shepherd")
    );

    Ok(())
}

#[test]
fn test_inheritance_builder_equality() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialEq for inheritance builders
    let dog_builder1 = dogs::table::builder().try_name("Buddy")?.breed("Labrador");

    let dog_builder2 = dogs::table::builder().try_name("Buddy")?.breed("Labrador");

    let dog_builder3 = dogs::table::builder().try_name("Max")?.breed("Labrador");

    let dog_builder4 = dogs::table::builder().try_name("Buddy")?.breed("Poodle");

    // Identical builders should be equal
    assert_eq!(dog_builder1, dog_builder2);

    // Different builders should not be equal
    assert_ne!(dog_builder1, dog_builder3);
    assert_ne!(dog_builder1, dog_builder4);
    assert_ne!(dog_builder3, dog_builder4);

    // The builders should also be equal to themselves
    assert_eq!(dog_builder1, dog_builder1);
    assert_eq!(dog_builder2, dog_builder2);
    assert_eq!(dog_builder3, dog_builder3);
    assert_eq!(dog_builder4, dog_builder4);

    Ok(())
}

#[test]
fn test_inheritance_builder_hash() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Test Hash for inheritance builders
    let dog_builder1 = dogs::table::builder().try_name("Buddy")?.breed("Labrador");

    let dog_builder2 = dogs::table::builder().try_name("Buddy")?.breed("Labrador");

    let dog_builder3 = dogs::table::builder().try_name("Max")?.breed("Labrador");

    let dog_builder4 = dogs::table::builder().try_name("Buddy")?.breed("Poodle");

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    dog_builder1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    dog_builder2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    dog_builder3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    dog_builder4.hash(&mut hasher4);
    let hash4 = hasher4.finish();

    // Identical builders should have the same hash
    assert_eq!(hash1, hash2);

    // Different builders should have different hashes
    assert_ne!(hash1, hash3);
    assert_ne!(hash1, hash4);
    assert_ne!(hash3, hash4);

    // Test that builders can be used as HashMap keys
    let mut map = HashMap::new();
    map.insert(dog_builder1.clone(), "buddy");
    map.insert(dog_builder3.clone(), "max");

    assert_eq!(map.get(&dog_builder2), Some(&"buddy"));
    assert_eq!(map.get(&dog_builder4), None);
    assert_eq!(map.get(&dog_builder3), Some(&"max"));

    Ok(())
}

#[test]
fn test_inheritance_builder_partial_ord() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialOrd implementation for TableBuilder
    let dog_builder1 = dogs::table::builder()
        .try_name("Buddy")?
        .try_breed("Labrador")?;

    let dog_builder2 = dogs::table::builder()
        .try_name("Buddy")?
        .try_breed("Labrador")?;

    let dog_builder3 = dogs::table::builder()
        .try_name("Max")?
        .try_breed("Labrador")?;

    let dog_builder4 = dogs::table::builder()
        .try_name("Buddy")?
        .try_breed("Poodle")?;

    // Identical builders should be equal
    assert_eq!(
        dog_builder1.partial_cmp(&dog_builder2),
        Some(std::cmp::Ordering::Equal)
    );

    // Different builders should have proper ordering
    assert_eq!(
        dog_builder1.partial_cmp(&dog_builder3),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        dog_builder3.partial_cmp(&dog_builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        dog_builder1.partial_cmp(&dog_builder4),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        dog_builder4.partial_cmp(&dog_builder1),
        Some(std::cmp::Ordering::Greater)
    );

    // Test Ord implementation
    assert_eq!(dog_builder1.cmp(&dog_builder2), std::cmp::Ordering::Equal);
    assert_eq!(dog_builder1.cmp(&dog_builder3), std::cmp::Ordering::Less);
    assert_eq!(dog_builder3.cmp(&dog_builder1), std::cmp::Ordering::Greater);
    assert_eq!(dog_builder1.cmp(&dog_builder4), std::cmp::Ordering::Less);
    assert_eq!(dog_builder4.cmp(&dog_builder1), std::cmp::Ordering::Greater);

    Ok(())
}
