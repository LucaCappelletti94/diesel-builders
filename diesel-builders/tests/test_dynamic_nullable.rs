//! Test to verify the dynamic retrieval of nullable columns.

mod shared;
mod shared_animals;
use diesel_builders::{TryGetDynamicColumn, prelude::*};
use shared_animals::*;

#[test]
fn test_dynamic_nullable_column_none() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Create an animal with NO description (None)
    // We only set the required name.
    let builder = animals::table::builder().try_name("Silent Animal")?;

    // Insert as nested implementation to get the structure that supports dynamic
    // access
    let nested_models = builder.insert_nested(&mut conn)?;

    // Define dynamic column key for description
    let dyn_desc_col: diesel_builders::DynColumn<String> = animals::description.into();

    // Try to get the dynamic column value
    let desc_val_ref = nested_models.try_get_dynamic_column_ref(dyn_desc_col)?;

    // This should hit the `return Ok(None)` path in `dynamic.rs`
    assert_eq!(desc_val_ref, None);

    // Also test the owned version
    let desc_val = nested_models.try_get_dynamic_column(dyn_desc_col)?;
    assert_eq!(desc_val, None);

    Ok(())
}

#[test]
fn test_dynamic_nullable_column_some() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    shared_animals::setup_animal_tables(&mut conn)?;

    // Create an animal WITH description
    let builder = animals::table::builder()
        .try_name("Loud Animal")?
        .try_description(Some("Roar".to_string()))?;

    let nested_models = builder.insert_nested(&mut conn)?;

    let dyn_desc_col: diesel_builders::DynColumn<String> = animals::description.into();

    let desc_val_ref = nested_models.try_get_dynamic_column_ref(dyn_desc_col)?;

    // Should be Some("Roar")
    assert_eq!(desc_val_ref, Some(&"Roar".to_string()));

    Ok(())
}
