//! Common utilities and shared table definitions for tests.

use std::convert::Infallible;

use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_builders::prelude::*;

/// Establish a `SQLite` connection with all necessary PRAGMAs enabled.
///
/// This function creates an in-memory `SQLite` database connection and enables
/// important PRAGMAs for testing:
/// - `foreign_keys = ON`: Enforces foreign key constraints
/// - `recursive_triggers = ON`: Allows triggers to be recursive
/// - `journal_mode = WAL`: Uses Write-Ahead Logging for better concurrency
#[allow(dead_code)]
pub fn establish_test_connection() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Enable foreign key constraints
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;

    // Enable recursive triggers
    diesel::sql_query("PRAGMA recursive_triggers = ON").execute(&mut conn)?;

    // Set journal mode to WAL for better performance
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(&mut conn)?;

    Ok(conn)
}

// ============================================================================
// Animals Table - Root table for all tests
// ============================================================================

diesel::table! {
    /// Animals table - root table representing all animals.
    animals (id) {
        /// Primary key of the animal.
        id -> Integer,
        /// The name of the animal.
        name -> Text,
        /// Optional description of the animal.
        description -> Nullable<Text>,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, Root, TableModel,
)]
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

/// SQL to create the animals table.
#[allow(dead_code)]
pub const CREATE_ANIMALS_TABLE: &str = "CREATE TABLE animals (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (name <> '' AND length(name) <= 100),
    description TEXT CHECK (description <> '' AND length(description) <= 500)
)";

// ============================================================================
// Dogs Table - Extends animals (left branch of DAG)
// ============================================================================

diesel::table! {
    /// Dogs table - extends animals via foreign key.
    dogs (id) {
        /// Primary key of the dog, foreign key to animals.id.
        id -> Integer,
        /// The breed of the dog.
        breed -> Text,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel, Decoupled,
)]
#[diesel(table_name = dogs)]
/// Model for the dogs table.
pub struct Dog {
    /// Primary key.
    id: i32,
    /// The breed of the dog.
    breed: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for dogs::table {
    type Ancestors = (animals::table,);
    type Root = animals::table;
}

/// SQL to create the dogs table.
#[allow(dead_code)]
pub const CREATE_DOGS_TABLE: &str = "CREATE TABLE dogs (
    id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id),
    breed TEXT NOT NULL
)";

// Declare singleton foreign key helper for `dogs.id` -> `animals` (single inheritance)
fpk!(dogs::id -> animals);

// ============================================================================
// Cats Table - Extends animals (right branch of DAG)
// ============================================================================

diesel::table! {
    /// Cats table - extends animals via foreign key.
    cats (id) {
        /// Primary key of the cat, foreign key to animals.id.
        id -> Integer,
        /// The color of the cat.
        color -> Text,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel, Decoupled,
)]
#[table_model(error = NewCatError)]
#[diesel(table_name = cats)]
/// Model for the cats table.
pub struct Cat {
    #[infallible]
    /// Primary key.
    id: i32,
    /// The color of the cat.
    color: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for cats::table {
    type Ancestors = (animals::table,);
    type Root = animals::table;
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash, thiserror::Error)]
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

/// SQL to create the cats table.
#[allow(dead_code)]
pub const CREATE_CATS_TABLE: &str = "CREATE TABLE cats (
    id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id),
    color TEXT NOT NULL CHECK (color <> '')
)";

// Declare singleton foreign key helper for `cats.id` -> `animals` (single inheritance)
fpk!(cats::id -> animals);

// ============================================================================
// Puppies Table - Extends dogs (for inheritance chain: animals -> dogs -> puppies)
// ============================================================================

diesel::table! {
    /// Puppies table - extends dogs via foreign key.
    puppies (id) {
        /// Primary key of the puppy, foreign key to dogs.id.
        id -> Integer,
        /// The age in months of the puppy.
        age_months -> Integer,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel, Decoupled,
)]
#[diesel(table_name = puppies)]
/// Model for the puppies table.
pub struct Puppy {
    /// Primary key.
    id: i32,
    /// The age in months of the puppy.
    age_months: i32,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for puppies::table {
    type Ancestors = (animals::table, dogs::table);
    type Root = animals::table;
}

/// SQL to create the puppies table.
#[allow(dead_code)]
pub const CREATE_PUPPIES_TABLE: &str = "CREATE TABLE puppies (
    id INTEGER PRIMARY KEY NOT NULL REFERENCES dogs(id),
    age_months INTEGER NOT NULL
)";

// ============================================================================
// Pets Table - Extends both dogs and cats (diamond inheritance for DAG tests)
// ============================================================================

diesel::table! {
    /// Pets table - extends both dogs and cats via foreign keys (diamond pattern).
    pets (id) {
        /// Primary key of the pet.
        id -> Integer,
        /// The owner name of the pet.
        owner_name -> Text,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel, Decoupled,
)]
#[diesel(table_name = pets)]
/// Model for the pets table.
pub struct Pet {
    /// Primary key.
    id: i32,
    /// The owner name of the pet.
    owner_name: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for pets::table {
    type Ancestors = (animals::table, dogs::table, cats::table);
    type Root = animals::table;
}

/// SQL to create the pets table (for DAG tests).
#[allow(dead_code)]
pub const CREATE_PETS_TABLE: &str = "CREATE TABLE pets (
    id INTEGER PRIMARY KEY NOT NULL,
    owner_name TEXT NOT NULL,
    FOREIGN KEY (id) REFERENCES dogs(id),
    FOREIGN KEY (id) REFERENCES cats(id)
)";

// Allow all tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(animals, dogs, cats, puppies, pets);
