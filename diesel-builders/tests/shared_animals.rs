//! Common utilities and shared table definitions for tests.

use diesel_builders::prelude::*;
use std::convert::Infallible;

/// Setups the animal hierarchy tables in the given `SQLite` connection.
///
/// # Errors
///
/// Returns an `Err` if any of the SQL DDL statements fail.
pub fn setup_animal_tables(
    conn: &mut diesel::SqliteConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    use diesel::RunQueryDsl;
    const CREATE_ANIMALS_TABLE: &str = "CREATE TABLE animals (
        id INTEGER PRIMARY KEY NOT NULL,
        name TEXT NOT NULL CHECK (name <> '' AND length(name) <= 100),
        description TEXT CHECK (description <> '' AND length(description) <= 500)
    )";

    const CREATE_DOGS_TABLE: &str = "CREATE TABLE dogs (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
        breed TEXT NOT NULL
    )";

    const CREATE_CATS_TABLE: &str = "CREATE TABLE cats (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id) ON DELETE CASCADE,
        color TEXT NOT NULL CHECK (color <> '')
    )";

    const CREATE_PUPPIES_TABLE: &str = "CREATE TABLE puppies (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES dogs(id) ON DELETE CASCADE,
        age_months INTEGER NOT NULL CHECK (age_months >= 0)
    )";

    const CREATE_PETS_TABLE: &str = "CREATE TABLE pets (
        id INTEGER PRIMARY KEY NOT NULL,
        owner_name TEXT NOT NULL,
        FOREIGN KEY (id) REFERENCES dogs(id) ON DELETE CASCADE,
        FOREIGN KEY (id) REFERENCES cats(id) ON DELETE CASCADE
    )";

    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_DOGS_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_CATS_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_PUPPIES_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_PETS_TABLE).execute(conn)?;

    Ok(())
}

#[derive(
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    AsChangeset,
    Insertable,
    PartialEq,
    TableModel,
)]
#[diesel(table_name = animals)]
#[table_model(error = NewAnimalError, surrogate_key)]
/// Model for the animals table.
pub struct Animal {
    /// Primary key.
    pub id: i32,
    /// The name of the animal.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
}

/// Error variants for `NewAnimal` validation.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum NewAnimalError {
    /// Name cannot be empty.
    #[error("Animal name cannot be empty")]
    NameEmpty,
    /// Name is too long (max 100 characters).
    #[error("Animal name cannot exceed 100 characters")]
    NameTooLong,
    /// Description cannot be empty when provided.
    #[error("Animal description cannot be empty when provided")]
    DescriptionEmpty,
    /// Description is too long (max 500 characters).
    #[error("Animal description cannot exceed 500 characters")]
    DescriptionTooLong,
}

impl From<std::convert::Infallible> for NewAnimalError {
    fn from(inf: std::convert::Infallible) -> Self {
        match inf {}
    }
}

/// Validation for animal name - non-empty, max 100 chars.
impl diesel_builders::ValidateColumn<animals::name>
    for <animals::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewAnimalError;
    type Borrowed = str;

    fn validate_column(value: &Self::Borrowed) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            return Err(NewAnimalError::NameEmpty);
        }
        if value.len() > 100 {
            return Err(NewAnimalError::NameTooLong);
        }

        Ok(())
    }
}

/// Validation for animal description - when Some, must be non-empty, max 500 chars.
impl diesel_builders::ValidateColumn<animals::description>
    for <animals::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewAnimalError;
    type Borrowed = str;

    fn validate_column(value: &Self::Borrowed) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            return Err(NewAnimalError::DescriptionEmpty);
        }
        if value.len() > 500 {
            return Err(NewAnimalError::DescriptionTooLong);
        }

        Ok(())
    }
}

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel, Clone)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals))]
/// Model for the dogs table.
pub struct Dog {
    /// Primary key.
    pub id: i32,
    /// The breed of the dog.
    #[table_model(default = "Unknown")]
    pub breed: String,
}

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(error = NewCatError, ancestors(animals))]
#[diesel(table_name = cats)]
/// Model for the cats table.
pub struct Cat {
    #[infallible]
    /// Primary key.
    pub id: i32,
    /// The color of the cat.
    pub color: String,
}

#[derive(Debug, PartialEq, thiserror::Error)]
/// Errors for `NewCat` validation.
pub enum NewCatError {
    /// Color cannot be empty.
    #[error("Color cannot be empty")]
    ColorEmpty,
}

impl From<Infallible> for NewCatError {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl diesel_builders::ValidateColumn<cats::color>
    for <cats::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewCatError;
    type Borrowed = str;

    fn validate_column(value: &Self::Borrowed) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            return Err(NewCatError::ColorEmpty);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
/// Errors for `NewPuppy` validation.
pub enum NewPuppyError {
    /// Age cannot be negative.
    #[error("Age cannot be negative")]
    NegativeAge,
}

impl From<Infallible> for NewPuppyError {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

impl diesel_builders::ValidateColumn<puppies::age_months>
    for <puppies::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewPuppyError;
    type Borrowed = i32;

    fn validate_column(value: &i32) -> Result<(), Self::Error> {
        if *value < 0 {
            return Err(NewPuppyError::NegativeAge);
        }
        Ok(())
    }
}

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel, Clone)]
#[diesel(table_name = puppies)]
#[table_model(error = NewPuppyError)]
#[table_model(ancestors(animals, dogs))]
/// Model for the puppies table.
pub struct Puppy {
    #[infallible]
    /// Primary key.
    pub id: i32,
    /// The age in months of the puppy.
    #[table_model(default = 6)]
    pub age_months: i32,
}

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel, Clone)]
#[diesel(table_name = pets)]
#[table_model(ancestors(animals, dogs, cats))]
/// Model for the pets table.
pub struct Pet {
    /// Primary key.
    pub id: i32,
    /// The owner name of the pet.
    pub owner_name: String,
}
