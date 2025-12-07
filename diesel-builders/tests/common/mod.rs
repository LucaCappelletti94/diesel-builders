//! Common utilities and shared table definitions for tests.

use std::{convert::Infallible, fmt::Display};

use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_builders::prelude::*;

/// Establish a SQLite connection with all necessary PRAGMAs enabled.
///
/// This function creates an in-memory SQLite database connection and enables
/// important PRAGMAs for testing:
/// - `foreign_keys = ON`: Enforces foreign key constraints
/// - `recursive_triggers = ON`: Allows triggers to be recursive
/// - `journal_mode = WAL`: Uses Write-Ahead Logging for better concurrency
#[allow(dead_code)]
pub fn establish_test_connection() -> Result<SqliteConnection, diesel::ConnectionError> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Enable foreign key constraints
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .expect("Failed to enable foreign keys");

    // Enable recursive triggers
    diesel::sql_query("PRAGMA recursive_triggers = ON")
        .execute(&mut conn)
        .expect("Failed to enable recursive triggers");

    // Set journal mode to WAL for better performance
    diesel::sql_query("PRAGMA journal_mode = WAL")
        .execute(&mut conn)
        .expect("Failed to set journal mode");

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
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    PartialOrd,
    GetColumn,
    Root,
    TableModel,
)]
#[diesel(table_name = animals)]
/// Model for the animals table.
pub struct Animal {
    /// Primary key.
    pub id: i32,
    /// The name of the animal.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
}

#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Insertable, MayGetColumn, HasTable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = animals)]
#[allow(clippy::option_option)]
/// Insertable model for the animals table.
pub struct NewAnimal {
    /// The name of the animal.
    pub name: Option<String>,
    /// Optional description (nullable column uses Option<Option<T>>).
    pub description: Option<Option<String>>,
}

/// Error variants for `NewAnimal` validation.
#[derive(Debug, PartialEq, PartialOrd, Eq, Clone)]
pub enum NewAnimalError {
    /// Name cannot be empty.
    NameEmpty,
    /// Name is too long (max 100 characters).
    NameTooLong,
    /// Description cannot be empty when provided.
    DescriptionEmpty,
    /// Description is too long (max 500 characters).
    DescriptionTooLong,
}

impl Display for NewAnimalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewAnimalError::NameEmpty => write!(f, "Animal name cannot be empty"),
            NewAnimalError::NameTooLong => {
                write!(f, "Animal name cannot exceed 100 characters")
            }
            NewAnimalError::DescriptionEmpty => {
                write!(f, "Animal description cannot be empty when provided")
            }
            NewAnimalError::DescriptionTooLong => {
                write!(f, "Animal description cannot exceed 500 characters")
            }
        }
    }
}

impl std::error::Error for NewAnimalError {}

impl From<std::convert::Infallible> for NewAnimalError {
    fn from(inf: std::convert::Infallible) -> Self {
        match inf {}
    }
}

/// Validation for animal name - non-empty, max 100 chars.
impl diesel_builders::TrySetColumn<animals::name> for NewAnimal {
    type Error = NewAnimalError;

    fn try_set_column(&mut self, value: String) -> Result<&mut Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(NewAnimalError::NameEmpty);
        }
        if value.len() > 100 {
            return Err(NewAnimalError::NameTooLong);
        }
        self.name = Some(value);
        Ok(self)
    }
}

/// Validation for animal description - when Some, must be non-empty, max 500 chars.
impl diesel_builders::TrySetColumn<animals::description> for NewAnimal {
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
        self.description = Some(value);
        Ok(self)
    }
}

impl InsertableTableModel for NewAnimal {
    type Error = NewAnimalError;
}

impl TableAddition for animals::table {
    type InsertableModel = NewAnimal;
    type Model = Animal;
    type PrimaryKeyColumns = (animals::id,);
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
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    PartialOrd,
    GetColumn,
    TableModel,
    Decoupled,
)]
#[diesel(table_name = dogs)]
/// Model for the dogs table.
pub struct Dog {
    /// Primary key.
    pub id: i32,
    /// The breed of the dog.
    pub breed: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for dogs::table {
    type Ancestors = (animals::table,);
    type Root = animals::table;
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    Insertable,
    MayGetColumn,
    SetColumn,
    HasTable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = dogs)]
/// Insertable model for the dogs table.
pub struct NewDog {
    /// Primary key.
    pub id: Option<i32>,
    /// The breed of the dog.
    pub breed: Option<String>,
}

impl TableAddition for dogs::table {
    type InsertableModel = NewDog;
    type Model = Dog;
    type PrimaryKeyColumns = (dogs::id,);
}

/// SQL to create the dogs table.
#[allow(dead_code)]
pub const CREATE_DOGS_TABLE: &str = "CREATE TABLE dogs (
    id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id),
    breed TEXT NOT NULL
)";

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
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    PartialOrd,
    GetColumn,
    TableModel,
    Decoupled,
)]
#[diesel(table_name = cats)]
/// Model for the cats table.
pub struct Cat {
    /// Primary key.
    pub id: i32,
    /// The color of the cat.
    pub color: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for cats::table {
    type Ancestors = (animals::table,);
    type Root = animals::table;
}

#[derive(
    Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Insertable, MayGetColumn, HasTable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = cats)]
/// Insertable model for the cats table.
pub struct NewCat {
    /// Primary key.
    pub id: Option<i32>,
    /// The color of the cat.
    pub color: Option<String>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum NewCatError {
    /// Color cannot be empty.
    ColorEmpty,
}

impl From<Infallible> for NewCatError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl Display for NewCatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewCatError::ColorEmpty => write!(f, "Color cannot be empty"),
        }
    }
}

impl core::error::Error for NewCatError {}

impl diesel_builders::TrySetColumn<cats::id> for NewCat {
    type Error = Infallible;

    fn try_set_column(&mut self, value: i32) -> Result<&mut Self, Self::Error> {
        self.id = Some(value);
        Ok(self)
    }
}

impl diesel_builders::TrySetColumn<cats::color> for NewCat {
    type Error = NewCatError;

    fn try_set_column(&mut self, value: String) -> Result<&mut Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(NewCatError::ColorEmpty);
        }
        self.color = Some(value);
        Ok(self)
    }
}

impl TableAddition for cats::table {
    type InsertableModel = NewCat;
    type Model = Cat;
    type PrimaryKeyColumns = (cats::id,);
}

impl InsertableTableModel for NewCat {
    type Error = NewCatError;
}

/// SQL to create the cats table.
#[allow(dead_code)]
pub const CREATE_CATS_TABLE: &str = "CREATE TABLE cats (
    id INTEGER PRIMARY KEY NOT NULL REFERENCES animals(id),
    color TEXT NOT NULL CHECK (color <> '')
)";

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
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    PartialOrd,
    GetColumn,
    TableModel,
    Decoupled,
)]
#[diesel(table_name = puppies)]
/// Model for the puppies table.
pub struct Puppy {
    /// Primary key.
    pub id: i32,
    /// The age in months of the puppy.
    pub age_months: i32,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for puppies::table {
    type Ancestors = (animals::table, dogs::table);
    type Root = animals::table;
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    Insertable,
    MayGetColumn,
    SetColumn,
    HasTable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = puppies)]
/// Insertable model for the puppies table.
pub struct NewPuppy {
    /// Primary key.
    pub id: Option<i32>,
    /// The age in months of the puppy.
    pub age_months: Option<i32>,
}

impl TableAddition for puppies::table {
    type InsertableModel = NewPuppy;
    type Model = Puppy;
    type PrimaryKeyColumns = (puppies::id,);
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
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    PartialOrd,
    GetColumn,
    TableModel,
    Decoupled,
)]
#[diesel(table_name = pets)]
/// Model for the pets table.
pub struct Pet {
    /// Primary key.
    pub id: i32,
    /// The owner name of the pet.
    pub owner_name: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for pets::table {
    type Ancestors = (animals::table, dogs::table, cats::table);
    type Root = animals::table;
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Hash,
    Insertable,
    MayGetColumn,
    SetColumn,
    HasTable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = pets)]
/// Insertable model for the pets table.
pub struct NewPet {
    /// Primary key.
    pub id: Option<i32>,
    /// The owner name of the pet.
    pub owner_name: Option<String>,
}

impl TableAddition for pets::table {
    type InsertableModel = NewPet;
    type Model = Pet;
    type PrimaryKeyColumns = (pets::id,);
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
