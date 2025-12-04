//! Test case for foreign key based inheritance where the dependencies
//! form a chain, with Animals being the root, Dogs extending Animals,
//! and Puppies extending Dogs.

use std::collections::HashMap;
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

#[test]
fn test_inheritance_chain_builder_equality() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialEq for inheritance chain builders
    let puppy_builder1 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(3);

    let puppy_builder2 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(3);

    let puppy_builder3 = puppies::table::builder()
        .try_name("Max")?
        .breed("Labrador")
        .age_months(3);

    let puppy_builder4 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Poodle")
        .age_months(3);

    let puppy_builder5 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(6);

    // Identical builders should be equal
    assert_eq!(puppy_builder1, puppy_builder2);

    // Different builders should not be equal
    assert_ne!(puppy_builder1, puppy_builder3);
    assert_ne!(puppy_builder1, puppy_builder4);
    assert_ne!(puppy_builder1, puppy_builder5);
    assert_ne!(puppy_builder3, puppy_builder4);
    assert_ne!(puppy_builder3, puppy_builder5);
    assert_ne!(puppy_builder4, puppy_builder5);

    // The builders should also be equal to themselves
    assert_eq!(puppy_builder1, puppy_builder1);
    assert_eq!(puppy_builder2, puppy_builder2);
    assert_eq!(puppy_builder3, puppy_builder3);
    assert_eq!(puppy_builder4, puppy_builder4);
    assert_eq!(puppy_builder5, puppy_builder5);

    Ok(())
}

#[test]
fn test_inheritance_chain_builder_hash() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Test Hash for inheritance chain builders
    let puppy_builder1 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(3);

    let puppy_builder2 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(3);

    let puppy_builder3 = puppies::table::builder()
        .try_name("Max")?
        .breed("Labrador")
        .age_months(3);

    let puppy_builder4 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Poodle")
        .age_months(3);

    let puppy_builder5 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(6);

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    puppy_builder1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    puppy_builder2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    puppy_builder3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    puppy_builder4.hash(&mut hasher4);
    let hash4 = hasher4.finish();

    let mut hasher5 = DefaultHasher::new();
    puppy_builder5.hash(&mut hasher5);
    let hash5 = hasher5.finish();

    // Identical builders should have the same hash
    assert_eq!(hash1, hash2);

    // Different builders should have different hashes
    assert_ne!(hash1, hash3);
    assert_ne!(hash1, hash4);
    assert_ne!(hash1, hash5);
    assert_ne!(hash3, hash4);
    assert_ne!(hash3, hash5);
    assert_ne!(hash4, hash5);

    // Test that builders can be used as HashMap keys
    let mut map = HashMap::new();
    map.insert(puppy_builder1.clone(), "buddy");
    map.insert(puppy_builder3.clone(), "max");
    map.insert(puppy_builder4.clone(), "poodle");

    assert_eq!(map.get(&puppy_builder2), Some(&"buddy"));
    assert_eq!(map.get(&puppy_builder5), None);
    assert_eq!(map.get(&puppy_builder3), Some(&"max"));
    assert_eq!(map.get(&puppy_builder4), Some(&"poodle"));

    Ok(())
}

#[test]
fn test_inheritance_chain_builder_partial_ord() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialOrd implementation for TableBuilder
    let puppy_builder1 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(12);

    let puppy_builder2 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(12);

    let puppy_builder3 = puppies::table::builder()
        .try_name("Max")?
        .breed("Labrador")
        .age_months(12);

    let puppy_builder4 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Poodle")
        .age_months(12);

    let puppy_builder5 = puppies::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .age_months(6);

    // Identical builders should be equal
    assert_eq!(
        puppy_builder1.partial_cmp(&puppy_builder2),
        Some(std::cmp::Ordering::Equal)
    );

    // Different builders should have proper ordering
    assert_eq!(
        puppy_builder1.partial_cmp(&puppy_builder3),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        puppy_builder3.partial_cmp(&puppy_builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        puppy_builder1.partial_cmp(&puppy_builder4),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        puppy_builder4.partial_cmp(&puppy_builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        puppy_builder1.partial_cmp(&puppy_builder5),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        puppy_builder5.partial_cmp(&puppy_builder1),
        Some(std::cmp::Ordering::Less)
    );

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Puppy that extends Dogs which extends Animals (chain inheritance)
    let builder = puppies::table::builder()
        .try_name("Serialized Puppy")?
        .breed("Beagle")
        .age_months(6);

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<puppies::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - age_months is the only field directly in NewPuppy
    assert_eq!(
        deserialized.may_get_column::<puppies::age_months>(),
        Some(&6)
    );

    Ok(())
}
