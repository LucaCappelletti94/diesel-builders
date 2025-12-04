//! Test case for foreign key based inheritance where the dependencies
//! form a directed acyclic graph (DAG).
//!
//! We have a root table Animals, which has two descendants Dogs and Cats.
//! Both Dogs and Cats extend Animals via foreign keys. Then, we have a table
//! Pets that extends both Dogs and Cats via foreign keys (diamond pattern).

mod common;
use std::collections::HashMap;

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
fn test_dag_builder_equality() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialEq for DAG builders
    let pet_builder1 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder2 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder3 = pets::table::builder()
        .try_name("Max")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder4 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Poodle")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder5 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Black")?
        .owner_name("Alice");

    let pet_builder6 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Bob");

    // Identical builders should be equal
    assert_eq!(pet_builder1, pet_builder2);

    // Different builders should not be equal
    assert_ne!(pet_builder1, pet_builder3);
    assert_ne!(pet_builder1, pet_builder4);
    assert_ne!(pet_builder1, pet_builder5);
    assert_ne!(pet_builder1, pet_builder6);

    // The builders should also be equal to themselves
    assert_eq!(pet_builder1, pet_builder1);
    assert_eq!(pet_builder2, pet_builder2);
    assert_eq!(pet_builder3, pet_builder3);
    assert_eq!(pet_builder4, pet_builder4);
    assert_eq!(pet_builder5, pet_builder5);
    assert_eq!(pet_builder6, pet_builder6);

    Ok(())
}

#[test]
fn test_dag_builder_hash() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Test Hash for DAG builders
    let pet_builder1 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder2 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder3 = pets::table::builder()
        .try_name("Max")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder4 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Poodle")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder5 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Black")?
        .owner_name("Alice");

    let pet_builder6 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Bob");

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    pet_builder1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    pet_builder2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    pet_builder3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    pet_builder4.hash(&mut hasher4);
    let hash4 = hasher4.finish();

    let mut hasher5 = DefaultHasher::new();
    pet_builder5.hash(&mut hasher5);
    let hash5 = hasher5.finish();

    let mut hasher6 = DefaultHasher::new();
    pet_builder6.hash(&mut hasher6);
    let hash6 = hasher6.finish();

    // Identical builders should have the same hash
    assert_eq!(hash1, hash2);

    // Different builders should have different hashes
    assert_ne!(hash1, hash3);
    assert_ne!(hash1, hash4);
    assert_ne!(hash1, hash5);
    assert_ne!(hash1, hash6);

    // Test that builders can be used as HashMap keys
    let mut map = HashMap::new();
    map.insert(pet_builder1.clone(), "alice_buddy");
    map.insert(pet_builder3.clone(), "alice_max");
    map.insert(pet_builder4.clone(), "alice_buddy_poodle");

    assert_eq!(map.get(&pet_builder2), Some(&"alice_buddy"));
    assert_eq!(map.get(&pet_builder5), None);
    assert_eq!(map.get(&pet_builder6), None);
    assert_eq!(map.get(&pet_builder3), Some(&"alice_max"));
    assert_eq!(map.get(&pet_builder4), Some(&"alice_buddy_poodle"));

    Ok(())
}

#[test]
fn test_dag_builder_partial_ord() -> Result<(), Box<dyn std::error::Error>> {
    // Test PartialOrd implementation for TableBuilder
    let pet_builder1 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder2 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder3 = pets::table::builder()
        .try_name("Max")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder4 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Poodle")
        .try_color("Brown")?
        .owner_name("Alice");

    let pet_builder5 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Black")?
        .owner_name("Alice");

    let pet_builder6 = pets::table::builder()
        .try_name("Buddy")?
        .breed("Labrador")
        .try_color("Brown")?
        .owner_name("Bob");

    // Identical builders should be equal
    assert_eq!(
        pet_builder1.partial_cmp(&pet_builder2),
        Some(std::cmp::Ordering::Equal)
    );

    // Different builders should have proper ordering
    assert_eq!(
        pet_builder1.partial_cmp(&pet_builder3),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        pet_builder3.partial_cmp(&pet_builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        pet_builder1.partial_cmp(&pet_builder4),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        pet_builder4.partial_cmp(&pet_builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        pet_builder1.partial_cmp(&pet_builder5),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        pet_builder5.partial_cmp(&pet_builder1),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        pet_builder1.partial_cmp(&pet_builder6),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        pet_builder6.partial_cmp(&pet_builder1),
        Some(std::cmp::Ordering::Greater)
    );

    // Test Ord implementation
    assert_eq!(pet_builder1.cmp(&pet_builder2), std::cmp::Ordering::Equal);
    assert_eq!(pet_builder1.cmp(&pet_builder3), std::cmp::Ordering::Less);
    assert_eq!(pet_builder3.cmp(&pet_builder1), std::cmp::Ordering::Greater);
    assert_eq!(pet_builder1.cmp(&pet_builder4), std::cmp::Ordering::Less);
    assert_eq!(pet_builder4.cmp(&pet_builder1), std::cmp::Ordering::Greater);
    assert_eq!(pet_builder1.cmp(&pet_builder5), std::cmp::Ordering::Greater);
    assert_eq!(pet_builder5.cmp(&pet_builder1), std::cmp::Ordering::Less);
    assert_eq!(pet_builder1.cmp(&pet_builder6), std::cmp::Ordering::Less);
    assert_eq!(pet_builder6.cmp(&pet_builder1), std::cmp::Ordering::Greater);

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
