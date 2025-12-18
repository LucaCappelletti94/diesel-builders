# Diesel Builders

[![CI](https://github.com/LucaCappelletti94/diesel-builders/workflows/Rust%20CI/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Security Audit](https://github.com/LucaCappelletti94/diesel-builders/workflows/Security%20Audit/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Codecov](https://codecov.io/gh/LucaCappelletti94/diesel-builders/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A type-safe builder pattern library for [Diesel](https://diesel.rs) handling complex table relationships (inheritance chains, DAGs, triangular dependencies) with compile-time guarantees for insertion order and referential integrity. It provides fluent APIs for getting/setting column values and associated builders and models, executing foreign key queries, and [`serde`](https://github.com/serde-rs/serde) support.

[Custom Diesel types](diesel-builders/tests/test_custom_type.rs) and [tables with multi-column primary keys](examples/composite_primary_keys.rs) are fully supported. The builder pattern works seamlessly with custom SQL and Rust types that implement the required Diesel traits (`AsExpression`, `FromSql`, `ToSql`).

The `TableModel` derive macro generates Diesel's `table!` macro, eliminating manual schema definitions.

## Installation

```toml
[dependencies]
diesel-builders = {git = "https://github.com/LucaCappelletti94/diesel-builders.git", branch = "main" }
```

## Supported Patterns

### 1. Simple Table (Base Case)

[A single table with no relationships](diesel-builders/tests/test_base_case.rs). This demonstrates the most basic usage of the builder pattern with type-safe column setters.

```rust
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = animals)]
#[table_model(surrogate_key)]
pub struct Animal {
    id: i32,
    name: String,
    description: Option<String>,
}

let mut conn = SqliteConnection::establish(":memory:")?;
diesel::sql_query("CREATE TABLE animals (id INTEGER PRIMARY KEY, name TEXT NOT NULL, description TEXT);").execute(&mut conn)?;

let animal = animals::table::builder()
    .name("Buddy")
    .description("A friendly dog".to_owned())
    .insert(&mut conn)?;

// You can load the table with `find`:
let loaded_animal: Animal = Animal::find(animal.id(), &mut conn)?;
assert_eq!(loaded_animal.name(), "Buddy");
// Delete the record
loaded_animal.delete(&mut conn)?;
// Check existence
assert!(!Animal::exists(loaded_animal.id(), &mut conn)?);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 2. Table Inheritance

[A linear inheritance chain](diesel-builders/tests/test_inheritance_chain.rs). Here, `dog_notes` in `Dog` uses `#[same_as(animals::description)]` to propagate its value up to `Animal`.

```rust
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = animals)]
#[table_model(surrogate_key)]
pub struct Animal {
    id: i32,
    name: String,
    #[table_model(default = "A really good boy")]
    description: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals))]
pub struct Dog {
    id: i32,
    breed: String,
    #[same_as(animals::description)]
    dog_notes: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = puppies)]
#[table_model(ancestors(animals, dogs))]
pub struct Puppy {
    id: i32,
    #[table_model(default = 6)]
    age_months: i32,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE animals (id INTEGER PRIMARY KEY, name TEXT NOT NULL, description TEXT DEFAULT 'A really good boy');").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE dogs (id INTEGER PRIMARY KEY REFERENCES animals(id) ON DELETE CASCADE, breed TEXT NOT NULL, dog_notes TEXT);").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE puppies (id INTEGER PRIMARY KEY REFERENCES dogs(id) ON DELETE CASCADE, age_months INTEGER NOT NULL DEFAULT 6);").execute(&mut conn)?;

let puppy = puppies::table::builder()
    .name("Buddy")
    .breed("Labrador")
    .dog_notes("A cute little puppy".to_owned())
    .age_months(3)
    .insert(&mut conn)?;

// You can load the table with `find`:
let loaded_puppy: Puppy = Puppy::find(puppy.id(), &mut conn)?;

// Access ancestor records
let animal: Animal = puppy.ancestor(&mut conn)?;
assert_eq!(animal.name(), "Buddy");
assert_eq!(animal.description().as_deref(), Some("A cute little puppy"));
let dog: Dog = puppy.ancestor(&mut conn)?;
assert_eq!(dog.breed(), "Labrador");
assert_eq!(dog.dog_notes().as_deref(), Some("A cute little puppy"));
assert_eq!(*puppy.age_months(), 3);

puppy.delete(&mut conn)?;
assert!(!Puppy::exists(puppy.id(), &mut conn)?);
assert!(!Dog::exists(puppy.id(), &mut conn)?);
assert!(!Animal::exists(puppy.id(), &mut conn)?);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 3. Directed Acyclic Graph (DAG)

[Multiple inheritance](diesel-builders/tests/test_dag.rs) where a child extends multiple parents. Pets extends Dogs and Cats, which both extend Animals.

```rust
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = animals)]
#[table_model(surrogate_key)]
pub struct Animal {
    id: i32,
    name: String,
    #[table_model(default = "No description")]
    description: String,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals))]
pub struct Dog {
    id: i32,
    breed: String,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = cats)]
#[table_model(ancestors(animals))]
pub struct Cat {
    id: i32,
    #[table_model(default = "All cats are orange")]
    color: String,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = pets)]
#[table_model(ancestors(animals, dogs, cats))]
pub struct Pet {
    id: i32,
    owner_name: String,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE animals (id INTEGER PRIMARY KEY, name TEXT NOT NULL, description TEXT NOT NULL DEFAULT 'No description');").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE dogs (id INTEGER PRIMARY KEY REFERENCES animals(id) ON DELETE CASCADE, breed TEXT NOT NULL);").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE cats (id INTEGER PRIMARY KEY REFERENCES animals(id) ON DELETE CASCADE, color TEXT NOT NULL DEFAULT 'All cats are orange');").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE pets (id INTEGER PRIMARY KEY, owner_name TEXT NOT NULL, FOREIGN KEY (id) REFERENCES dogs(id) ON DELETE CASCADE, FOREIGN KEY (id) REFERENCES cats(id) ON DELETE CASCADE);").execute(&mut conn)?;

let pet = pets::table::builder()
    .name("Bellerophon")
    .breed("Hybrid Orange-Labrador")
    .color("Orange")
    .owner_name("Alice Smith")
    .insert(&mut conn)?;

let animal: Animal = pet.ancestor(&mut conn)?;
assert_eq!(animal.name(), "Bellerophon");
let dog: Dog = pet.ancestor(&mut conn)?;
assert_eq!(dog.breed(), "Hybrid Orange-Labrador");
let cat: Cat = pet.ancestor(&mut conn)?;
assert_eq!(cat.color(), "Orange");

pet.delete(&mut conn)?;
assert!(!Pet::exists(pet.id(), &mut conn)?);
assert!(!Dog::exists(pet.id(), &mut conn)?);
assert!(!Cat::exists(pet.id(), &mut conn)?);
assert!(!Animal::exists(pet.id(), &mut conn)?);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 4. Mandatory Triangular Relation

[A complex pattern](diesel-builders/tests/test_mandatory_triangular_relation.rs) where Child extends Parent and references Mandatory, and Mandatory also references Parent. The `#[mandatory]` attribute ensures atomic creation. Insertion order: Parent → Mandatory → Child.

**Horizontal Same-As**: Like Vertical Same-As, but propagates values from referenced tables via foreign keys. Here, `remote_mandatory_field` mirrors `mandatory_table::mandatory_field` via `HorizontalKey`.

```rust
use diesel_builders::prelude::*;

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = parent_table)]
#[table_model(surrogate_key)]
pub struct Parent {
    id: i32,
    parent_field: String,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = mandatory_table)]
#[table_model(surrogate_key)]
pub struct Mandatory {
    id: i32,
    parent_id: i32,
    #[table_model(default = "Default mandatory")]
    mandatory_field: String,
}

diesel::allow_tables_to_appear_in_same_query!(parent_table, mandatory_table);
fpk!(mandatory_table::parent_id -> parent_table);
unique_index!(mandatory_table::id, mandatory_table::mandatory_field);
unique_index!(mandatory_table::id, mandatory_table::parent_id);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(ancestors = parent_table)]
pub struct Child {
    #[same_as(mandatory_table::parent_id)]
    id: i32,
    #[mandatory(mandatory_table)]
    mandatory_id: i32,
    child_field: String,
    #[same_as(mandatory_table::mandatory_field)]
    remote_mandatory_field: Option<String>,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query("CREATE TABLE parent_table (id INTEGER PRIMARY KEY NOT NULL, parent_field TEXT NOT NULL);").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE mandatory_table (id INTEGER PRIMARY KEY NOT NULL, parent_id INTEGER NOT NULL REFERENCES parent_table(id), mandatory_field TEXT NOT NULL DEFAULT 'Default mandatory', UNIQUE(id, mandatory_field), UNIQUE(id, parent_id));").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE child_table (id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id), mandatory_id INTEGER NOT NULL REFERENCES mandatory_table(id), child_field TEXT NOT NULL, remote_mandatory_field TEXT, FOREIGN KEY (mandatory_id, id) REFERENCES mandatory_table(id, parent_id), FOREIGN KEY (mandatory_id, remote_mandatory_field) REFERENCES mandatory_table(id, mandatory_field));").execute(&mut conn)?;

// Create Child with associated Mandatory (which automatically creates Parent)
let child = child_table::table::builder()
    .parent_field("Parent value")
    .child_field("Child value")
    .mandatory(mandatory_table::table::builder().mandatory_field("Mandatory value"))
    .insert(&mut conn)?;

// Access the associated Mandatory record
let mandatory: Mandatory = child.mandatory(&mut conn)?;
assert_eq!(mandatory.mandatory_field, "Mandatory value");
let mandatory_parent: Parent = mandatory.parent(&mut conn)?;
// Access the associated Parent record
let parent: Parent = child.ancestor(&mut conn)?;
assert_eq!(parent.parent_field(), "Parent value");
assert_eq!(parent, mandatory_parent);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 5. Discretionary Triangular Relation

[Similar to mandatory](diesel-builders/tests/test_discretionary_triangular_relation.rs), but Child can reference *any* Discretionary record. Use `try_discretionary()` (new record) or `try_discretionary_model()` (existing).

**Horizontal Same-As**: `remote_discretionary_field` mirrors `discretionary_table::discretionary_field`.

```rust
use diesel_builders::prelude::*;

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = parent_table)]
#[table_model(surrogate_key)]
pub struct Parent {
    id: i32,
    parent_field: String,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = discretionary_table)]
#[table_model(surrogate_key)]
pub struct Discretionary {
    id: i32,
    parent_id: i32,
    discretionary_field: String,
}

diesel::allow_tables_to_appear_in_same_query!(parent_table, discretionary_table);
fpk!(discretionary_table::parent_id -> parent_table);
unique_index!(discretionary_table::id, discretionary_table::discretionary_field);
unique_index!(discretionary_table::id, discretionary_table::parent_id);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_with_discretionary_table)]
#[table_model(ancestors = parent_table)]
pub struct Child {
    #[same_as(discretionary_table::parent_id)]
    id: i32,
    #[discretionary(discretionary_table)]
    discretionary_id: i32,
    child_field: String,
    #[same_as(discretionary_table::discretionary_field)]
    remote_discretionary_field: Option<String>,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query("CREATE TABLE parent_table (id INTEGER PRIMARY KEY NOT NULL, parent_field TEXT NOT NULL);").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE discretionary_table (id INTEGER PRIMARY KEY NOT NULL, parent_id INTEGER NOT NULL REFERENCES parent_table(id), discretionary_field TEXT, UNIQUE(id, discretionary_field));").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE child_with_discretionary_table (id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id), discretionary_id INTEGER NOT NULL REFERENCES discretionary_table(id), child_field TEXT NOT NULL, remote_discretionary_field TEXT, FOREIGN KEY (discretionary_id, remote_discretionary_field) REFERENCES discretionary_table(id, discretionary_field));").execute(&mut conn)?;

// Example 1: Using a builder to create a new Discretionary record
let child = child_with_discretionary_table::table::builder()
    .parent_field("Parent value")
    .child_field("Child value")
    .discretionary(discretionary_table::table::builder().discretionary_field("New discretionary"))
    .insert(&mut conn)?;

let discretionary = child.discretionary(&mut conn)?;
let parent: Parent = child.ancestor(&mut conn)?;
let discretionary_parent: Parent = discretionary.parent(&mut conn)?;
assert_eq!(parent, discretionary_parent);

// Example 2: Using an existing Discretionary model
let child2 = child_with_discretionary_table::table::builder()
    .parent_field("Different parent")
    .child_field("Child 2 value")
    .discretionary_model(&discretionary)
    .insert(&mut conn)?;

let discretionary2: Discretionary = child2.discretionary(&mut conn)?;
assert_eq!(discretionary2, discretionary);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 6. Validation with Check Constraints

[Custom validation](diesel-builders/tests/test_inheritance.rs) via `ValidateColumn` mirrors SQL CHECK constraints. Supports runtime and compile-time (via `const_validator`) checks.

```rust
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = users)]
#[table_model(error = UserError, surrogate_key)]
pub struct User {
    id: i32,
    #[infallible]
    username: String,
    #[table_model(default = 18)]
    age: i32,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum UserError {
    #[error("Age must be at least 18")]
    AgeTooYoung,
}

// Compile-time validation for age (including default value)
#[diesel_builders_derive::const_validator]
impl ValidateColumn<users::age> for <users::table as TableExt>::NewValues {
    type Error = UserError;
    type Borrowed = i32;
    
    fn validate_column(value: &i32) -> Result<(), Self::Error> {
        if *value < 18 { return Err(UserError::AgeTooYoung); }
        Ok(())
    }
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL, age INTEGER NOT NULL DEFAULT 18 CHECK (age >= 18));").execute(&mut conn)?;

// Valid insertion using default age
let user = users::table::builder()
    .username("alice")
    .insert(&mut conn)?;

assert_eq!(user.username(), "alice");
assert_eq!(*user.age(), 18); // Default value was validated at compile time

// Valid insertion with explicit age
let user2 = users::table::builder()
    .username("bob")
    .try_age(25)?
    .insert(&mut conn)?;

assert_eq!(*user2.age(), 25);

// Runtime validation errors
let result = users::table::builder().try_age(7);  // Error: AgeTooYoung
assert_eq!(result.unwrap_err(), UserError::AgeTooYoung);

// Compile-time validated default prevents this at compile time:
// #[table_model(default = 16)]  // Would fail to compile!
// age: i32,

Ok::<(), Box<dyn std::error::Error>>(())
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
