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
        id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id),
        breed TEXT NOT NULL
    )";

    const CREATE_CATS_TABLE: &str = "CREATE TABLE cats (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id),
        color TEXT NOT NULL CHECK (color <> '')
    )";

    const CREATE_PUPPIES_TABLE: &str = "CREATE TABLE puppies (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES dogs(id),
        age_months INTEGER NOT NULL
    )";

    const CREATE_PETS_TABLE: &str = "CREATE TABLE pets (
        id INTEGER PRIMARY KEY NOT NULL,
        owner_name TEXT NOT NULL,
        FOREIGN KEY (id) REFERENCES dogs(id),
        FOREIGN KEY (id) REFERENCES cats(id)
    )";

    diesel::sql_query(CREATE_ANIMALS_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_DOGS_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_CATS_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_PUPPIES_TABLE).execute(conn)?;
    diesel::sql_query(CREATE_PETS_TABLE).execute(conn)?;

    Ok(())
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = animals)]
#[table_model(error = NewAnimalError, surrogate_key)]
/// Model for the animals table.
pub struct Animal {
    /// Primary key.
    id: i32,
    /// The name of the animal.
    name: String,
    /// Optional description.
    description: Option<String>,
}

/// Error variants for `NewAnimal` validation.
#[derive(Debug, PartialEq, PartialOrd, Eq, Clone, thiserror::Error)]
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
impl diesel_builders::TrySetColumn<animals::name>
    for <animals::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewAnimalError;

    fn try_set_column(&mut self, value: String) -> Result<&mut Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(NewAnimalError::NameEmpty);
        }
        if value.len() > 100 {
            return Err(NewAnimalError::NameTooLong);
        }
        self.set_column_unchecked::<animals::name>(value);
        Ok(self)
    }
}

/// Validation for animal description - when Some, must be non-empty, max 500 chars.
impl diesel_builders::TrySetColumn<animals::description>
    for <animals::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewAnimalError;

    fn try_set_column(&mut self, value: Option<String>) -> Result<&mut Self, Self::Error> {
        if let Some(ref desc) = value {
            if desc.trim().is_empty() {
                return Err(NewAnimalError::DescriptionEmpty);
            }
            if desc.len() > 500 {
                return Err(NewAnimalError::DescriptionTooLong);
            }
        }
        self.set_column_unchecked::<animals::description>(value);
        Ok(self)
    }
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals))]
/// Model for the dogs table.
pub struct Dog {
    /// Primary key.
    id: i32,
    /// The breed of the dog.
    breed: String,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[table_model(error = NewCatError, ancestors(animals))]
#[diesel(table_name = cats)]
/// Model for the cats table.
pub struct Cat {
    #[infallible]
    /// Primary key.
    id: i32,
    /// The color of the cat.
    color: String,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash, thiserror::Error)]
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

impl diesel_builders::TrySetColumn<cats::color>
    for <cats::table as diesel_builders::TableExt>::NewValues
{
    type Error = NewCatError;

    fn try_set_column(&mut self, value: String) -> Result<&mut Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(NewCatError::ColorEmpty);
        }
        self.set_column_unchecked::<cats::color>(value);
        Ok(self)
    }
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = puppies)]
#[table_model(ancestors(animals, dogs))]
/// Model for the puppies table.
pub struct Puppy {
    /// Primary key.
    id: i32,
    /// The age in months of the puppy.
    age_months: i32,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = pets)]
#[table_model(ancestors(animals, dogs, cats))]
/// Model for the pets table.
pub struct Pet {
    /// Primary key.
    id: i32,
    /// The owner name of the pet.
    owner_name: String,
}

// Allow all tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(animals, dogs, cats, puppies, pets);
