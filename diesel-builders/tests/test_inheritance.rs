//! Test case for foreign key based inheritance where Dogs extends
//! Animals. The primary key of Dogs is a foreign key to the primary key
//! of Animals.

mod shared;
mod shared_animals;
use diesel::prelude::*;
use diesel_builders::prelude::*;
use shared_animals::*;

#[test]
fn test_dog_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // We create an animal without a dog entry - demonstrating Root derive
    let animal = animals::table::builder().try_name("Generic Animal")?.insert(&mut conn)?;

    let loaded_animal: Animal = Animal::find(animal.id(), &mut conn)?;
    assert_eq!(loaded_animal, animal.clone());

    // Test TableModel derive - accessing primary key via GetColumn
    assert_eq!(loaded_animal.id(), animal.id());

    // Now create a dog (which also creates an animal entry via inheritance)
    let dog_builder = dogs::table::try_builder()?.try_name("Max")?;

    // Test generated helper traits - fluent API for setting columns
    let dog_builder = dog_builder.breed("Golden Retriever");

    // Test MayGetColumn derive - verifying builder state before insertion
    let breed_value = dog_builder.may_get_column_ref::<dogs::breed>();
    assert_eq!(breed_value, Some(&"Golden Retriever".to_string()));

    let dog = dog_builder.insert(&mut conn)?;

    assert_eq!(dog.breed(), "Golden Retriever");

    // Verify the dog can be queried
    let queried_dog: Dog = Dog::find(dog.id(), &mut conn)?;
    assert_eq!(dog, queried_dog);

    let loaded_animal: Animal = dog.ancestor(&mut conn)?;

    let loaded_dog: Dog = Dog::find(dog.id(), &mut conn)?;

    // Test GetColumn derive on both parent and child models
    assert_eq!(loaded_animal.id(), dog.id());
    assert_eq!(loaded_animal.name(), "Max");
    assert_eq!(loaded_animal.description().as_deref(), Some("A generic dog"));
    assert_eq!(loaded_dog.breed(), "Golden Retriever");
    assert_eq!(loaded_dog, dog);

    // Test delete cascade - deleting dog should cascade delete from animals
    // table
    let dog_id = dog.id();
    let deleted_rows = dog.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify the dog is deleted
    assert!(!Dog::exists(dog_id, &mut conn)?);

    // Verify the associated animal is also deleted due to CASCADE
    assert!(!Animal::exists(dog_id, &mut conn)?);

    // The standalone animal should still exist
    assert!(Animal::exists(animal.id(), &mut conn)?);

    Ok(())
}

#[test]
fn test_cat_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    setup_animal_tables(&mut conn)?;

    // Now create a cat (which also creates an animal entry via inheritance)
    let cat_builder = cats::table::builder().try_name("Whiskers")?.try_color("Orange")?;

    // Test MayGetColumn derive on builder to verify state
    let color_value = cat_builder.may_get_column_ref::<cats::color>();
    assert_eq!(color_value, Some(&"Orange".to_string()));

    let cat = cat_builder.insert(&mut conn)?;

    assert_eq!(cat.color(), "Orange");

    // Verify the cat can be queried
    let queried_cat: Cat = Cat::find(cat.id(), &mut conn)?;
    assert_eq!(cat, queried_cat);

    let loaded_animal: Animal = cat.ancestor(&mut conn)?;

    let loaded_cat: Cat = Cat::find(cat.id(), &mut conn)?;

    // Test GetColumn derive - type-safe column access on both models
    assert_eq!(loaded_animal.id(), cat.id());
    assert_eq!(loaded_animal.name(), "Whiskers");
    assert_eq!(loaded_cat.color(), "Orange");
    assert_eq!(loaded_cat, cat);

    // Test delete cascade for cat
    let cat_id = cat.id();
    let deleted_rows = cat.delete(&mut conn)?;
    assert_eq!(deleted_rows, 1);

    // Verify the cat is deleted
    assert!(!Cat::exists(cat_id, &mut conn)?);

    // Verify the associated animal is also deleted due to CASCADE
    assert!(!Animal::exists(cat_id, &mut conn)?);

    Ok(())
}

#[test]
fn test_nested_method() -> Result<(), Box<dyn std::error::Error>> {
    use diesel_builders::table_model::TableModel;

    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Create a dog (inherits from Animal)
    let dog: Dog = dogs::table::try_builder()?
        .try_name("NestedDog")?
        .breed("NestedBreed")
        .insert(&mut conn)?;

    // Load nested model using .nested()
    // For Dog, NestedModel is (Animal, (Dog,))
    let nested: diesel_builders::NestedModel<dogs::table> = dog.nested(&mut conn)?;

    // Verify contents
    assert_eq!(nested.get_column::<animals::name>(), "NestedDog");
    assert_eq!(nested.get_column::<dogs::breed>(), "NestedBreed");

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder for a Dog that extends Animals
    let builder = dogs::table::try_builder()?.try_name("Serialized Dog")?.breed("German Shepherd");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<dogs::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match - breed is the only field directly in NewDog
    assert_eq!(
        deserialized.may_get_column_ref::<dogs::breed>().map(String::as_str),
        Some("German Shepherd")
    );

    Ok(())
}

#[test]
fn test_dynamic_column_setting_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Create a dog using dynamic column setting
    let dyn_breed_column = dogs::breed.into();
    let dyn_name_column = animals::name.into();

    let dog = dogs::table::try_builder()?
        .try_set_dynamic_column(dyn_name_column, &"Dynamic Dog".to_owned())?
        .try_set_dynamic_column(dyn_breed_column, &"Dynamic Breed".to_owned())?
        .insert(&mut conn)?;

    // Load the ancestor to check name
    let loaded_animal: Animal = dog.ancestor(&mut conn)?;
    assert_eq!(loaded_animal.name(), "Dynamic Dog");
    assert_eq!(dog.breed(), "Dynamic Breed");

    // Verify via query
    let queried_dog: Dog = Dog::find(dog.id(), &mut conn)?;
    let queried_animal: Animal = queried_dog.ancestor(&mut conn)?;
    assert_eq!(queried_animal.name(), "Dynamic Dog");
    assert_eq!(queried_dog.breed(), "Dynamic Breed");

    // Test Variadic retrieval with insert_nested
    let dog_builder = dogs::table::try_builder()?
        .try_set_dynamic_column(dyn_name_column, &"Dynamic Dog 2".to_owned())?
        .try_set_dynamic_column(dyn_breed_column, &"Dynamic Breed 2".to_owned())?;

    let nested_dog = dog_builder.insert_nested(&mut conn)?;

    let cols = (dyn_name_column, (dyn_breed_column,));
    let (name_opt, (breed_opt,)) = nested_dog.try_get_dynamic_columns_ref(cols)?;
    assert_eq!(name_opt.map(String::as_str), Some("Dynamic Dog 2"));
    assert_eq!(breed_opt.map(String::as_str), Some("Dynamic Breed 2"));

    Ok(())
}

#[test]
fn test_get_model_ext_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    let dog_builder = dogs::table::try_builder()?.try_name("Rex")?.breed("Labrador");
    let nested_dog = dog_builder.insert_nested(&mut conn)?;

    // nested_dog should allow accessing both Dog and Animal models
    let dog = nested_dog.get_model_ref::<dogs::table>();
    let animal = nested_dog.get_model_ref::<animals::table>();

    assert_eq!(dog.breed(), "Labrador");
    assert_eq!(animal.name(), "Rex");
    assert_eq!(dog.id(), animal.id());

    // Test owned variant
    let dog_owned = nested_dog.get_model::<dogs::table>();
    assert_eq!(dog_owned.breed(), "Labrador");

    Ok(())
}

/// Failed descendant insertion preserves existing rows inside and outside a
/// transaction.
#[test]
fn test_insert_rolls_back_ancestors() {
    for nested in [false, true] {
        for outer_transaction in [false, true] {
            let mut conn = shared::establish_connection().unwrap();
            setup_animal_tables(&mut conn).unwrap();
            diesel::sql_query("CREATE UNIQUE INDEX dogs_unique_breed ON dogs(breed)")
                .execute(&mut conn)
                .unwrap();
            dogs::table::try_builder()
                .unwrap()
                .try_name("Existing")
                .unwrap()
                .breed("Duplicate")
                .insert(&mut conn)
                .unwrap();

            let operation = |conn: &mut diesel::SqliteConnection| -> diesel::QueryResult<()> {
                let builder = dogs::table::try_builder()
                    .unwrap()
                    .try_name("Rejected")
                    .unwrap()
                    .breed("Duplicate");
                let result = if nested {
                    builder.insert_nested(conn).map(|_| ())
                } else {
                    builder.insert(conn).map(|_| ())
                };
                assert!(matches!(
                    result,
                    Err(diesel_builders::BuilderError::Diesel(
                        diesel::result::Error::DatabaseError(
                            diesel::result::DatabaseErrorKind::UniqueViolation,
                            _
                        )
                    ))
                ));
                assert_eq!(animals::table.count().get_result::<i64>(conn)?, 1);
                assert_eq!(dogs::table.count().get_result::<i64>(conn)?, 1);
                dogs::table::try_builder()
                    .unwrap()
                    .try_name("Accepted")
                    .unwrap()
                    .breed("Distinct")
                    .insert(conn)
                    .unwrap();
                assert_eq!(animals::table.count().get_result::<i64>(conn)?, 2);
                Ok(())
            };
            if outer_transaction {
                conn.transaction(operation).unwrap();
            } else {
                operation(&mut conn).unwrap();
            }
            assert_eq!(dogs::table.count().get_result::<i64>(&mut conn).unwrap(), 2);
        }
    }
}
