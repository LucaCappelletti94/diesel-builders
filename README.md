# Diesel Builders

[![CI](https://github.com/LucaCappelletti94/diesel-builders/workflows/Rust%20CI/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Security Audit](https://github.com/LucaCappelletti94/diesel-builders/workflows/Security%20Audit/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Codecov](https://codecov.io/gh/LucaCappelletti94/diesel-builders/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A type-safe builder pattern library for [Diesel](https://diesel.rs) that handles complex table relationships including arbitrary inheritance (e.g. chains, DAG dependencies), foreign keys, and both mandatory and discretionary triangular dependencies. Diesel Builders provides compile-time guarantees for proper insertion order and referential integrity in databases with complex schemas.

The `TableModel` derive macro automatically generates the `table!` macro invocation for Diesel, eliminating the need to manually define your table schema. This means you only need to define your struct once with the `#[diesel(table_name = ...)]` attribute, and the macro handles the rest - no duplicate `table!` declarations required.

It additionally offers fluent APIs for getting/setting column values and associated builders and models, and [`serde`](https://github.com/serde-rs/serde) support.

This library is transparent in terms of backends and should work for any Diesel backend. In the README and tests, we use `SQLite` for simplicity.

## Installation

Add this to your `Cargo.toml`:

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
diesel::sql_query(
    "CREATE TABLE animals (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT
    );",
).execute(&mut conn)?;

let animal = animals::table::builder()
    .name("Buddy")
    .description("A friendly dog".to_owned())
    .insert(&mut conn)?;

Ok::<(), Box<dyn std::error::Error>>(())
```

### 2. Table Inheritance

[Tables extending a parent table](diesel-builders/tests/test_inheritance.rs) via foreign key on the primary key. When inserting into a child table, the builder automatically creates the parent record and ensures proper referential integrity. The `ancestors` attribute in `#[table_model]` declares the inheritance relationship.

```mermaid
erDiagram
    ANIMALS ||--o| DOGS : extends
    ANIMALS {
        int id PK
        string name
        string description
    }
    DOGS {
        int id PK,FK
        string breed
    }
```

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

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals))]
pub struct Dog {
    id: i32,
    breed: String,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query(
    "CREATE TABLE animals (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE dogs (
        id INTEGER PRIMARY KEY REFERENCES animals(id),
        breed TEXT NOT NULL
    );"
).execute(&mut conn)?;

let dog = dogs::table::builder()
    .name("Max")
    .description("A playful puppy".to_owned())
    .breed("Golden Retriever")
    .insert(&mut conn)?;

let animal: Animal = dog.ancestor(&mut conn)?;
assert_eq!(animal.name(), "Max");
assert_eq!(animal.description().as_deref(), Some("A playful puppy"));

Ok::<(), Box<dyn std::error::Error>>(())
```

### 3. Inheritance Chain

[A linear inheritance chain](diesel-builders/tests/test_inheritance_chain.rs) where each table extends exactly one parent. Puppies extends Dogs, which extends Animals. The builder automatically determines and enforces the correct insertion order through the dependency graph. Insertion order: Animals → Dogs → Puppies.

```mermaid
erDiagram
    ANIMALS ||--o| DOGS : extends
    DOGS ||--o| PUPPIES : extends
    ANIMALS {
        int id PK
        string name
        string description
    }
    DOGS {
        int id PK,FK
        string breed
    }
    PUPPIES {
        int id PK,FK
        int age_months
    }
```

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

diesel::sql_query(
    "CREATE TABLE animals (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE dogs (
        id INTEGER PRIMARY KEY REFERENCES animals(id),
        breed TEXT NOT NULL
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE puppies (
        id INTEGER PRIMARY KEY REFERENCES dogs(id),
        age_months INTEGER NOT NULL
    );"
).execute(&mut conn)?;

let puppy = puppies::table::builder()
    .name("Buddy")
    .description("A cute little puppy".to_owned())
    .breed("Labrador")
    .age_months(3)
    .insert(&mut conn)?;

let animal: Animal = puppy.ancestor(&mut conn)?;
assert_eq!(animal.name(), "Buddy");
let dog: Dog = puppy.ancestor(&mut conn)?;
assert_eq!(dog.breed(), "Labrador");
assert_eq!(*puppy.age_months(), 3);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 4. Directed Acyclic Graph (DAG)

[Multiple inheritance](diesel-builders/tests/test_dag.rs) where a child table extends multiple parent tables. Pets extends both Dogs and Cats, which both extend Animals. The builder automatically resolves the dependency graph and inserts records in the correct order, ensuring all foreign key constraints are satisfied. Insertion order: Animals → Dogs → Cats → Pets.

Of course, in this scenario, your pet is a dog-cat hybrid! Everything is possible with CRISPR-Cas9 these days.

```mermaid
erDiagram
    ANIMALS ||--o| DOGS : extends
    ANIMALS ||--o| CATS : extends
    DOGS ||--o| PETS : extends
    CATS ||--o| PETS : extends
    ANIMALS {
        int id PK
        string name
        string description
    }
    DOGS {
        int id PK,FK
        string breed
    }
    CATS {
        int id PK,FK
        string color
    }
    PETS {
        int id PK,FK
        string owner_name
    }
```

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

diesel::sql_query(
    "CREATE TABLE animals (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        description TEXT NOT NULL
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE dogs (
        id INTEGER PRIMARY KEY REFERENCES animals(id),
        breed TEXT NOT NULL
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE cats (
        id INTEGER PRIMARY KEY REFERENCES animals(id),
        color TEXT NOT NULL
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE pets (
        id INTEGER PRIMARY KEY,
        owner_name TEXT NOT NULL,
        FOREIGN KEY (id) REFERENCES dogs(id),
        FOREIGN KEY (id) REFERENCES cats(id)
    );"
).execute(&mut conn)?;

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

Ok::<(), Box<dyn std::error::Error>>(())
```

### 5. Mandatory Triangular Relation

[A complex pattern](diesel-builders/tests/test_mandatory_triangular_relation.rs) where Child extends Parent and also references Mandatory, with the constraint that the Mandatory record must also reference the same Parent record (enforcing `Child.mandatory_id == Mandatory.parent_id == Parent.id`). The builder uses the `#[mandatory]` attribute and automatically creates both Child and its related Mandatory record atomically, ensuring referential consistency. Foreign key relationships are declared via `fk!` macro, with composite indices via `index!` macro. Insertion order: Parent → Mandatory → Child.

```mermaid
erDiagram
    PARENT_TABLE ||--o{ MANDATORY_TABLE : references
    PARENT_TABLE ||--o| CHILD_TABLE : extends
    MANDATORY_TABLE ||--|| CHILD_TABLE : "references"
    PARENT_TABLE {
        int id PK
        string parent_field
    }
    MANDATORY_TABLE {
        int id PK
        int parent_id FK "Must match Child.id"
        string mandatory_field
    }
    CHILD_TABLE {
        int id PK,FK "Inherits from Parent"
        int mandatory_id FK "References Mandatory"
        string child_field
        string remote_mandatory_field FK
    }
```

```rust
use diesel_builders::prelude::*;
use std::convert::Infallible;

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
    mandatory_field: Option<String>,
}

diesel::allow_tables_to_appear_in_same_query!(parent_table, mandatory_table);
fpk!(mandatory_table::parent_id -> parent_table);
index!(mandatory_table::id, mandatory_table::mandatory_field);
index!(mandatory_table::id, mandatory_table::parent_id);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(ancestors = parent_table)]
pub struct Child {
    id: i32,
    #[mandatory(mandatory_table)]
    mandatory_id: i32,
    child_field: String,
    remote_mandatory_field: Option<String>,
}

fk!((child_table::mandatory_id, child_table::id) -> (mandatory_table::id, mandatory_table::parent_id));
fk!((child_table::mandatory_id, child_table::remote_mandatory_field) -> (mandatory_table::id, mandatory_table::mandatory_field));

impl diesel_builders::HorizontalKey for child_table::mandatory_id {
    type HostColumns = (
        child_table::id,
        child_table::remote_mandatory_field,
    );
    type ForeignColumns = (mandatory_table::parent_id, mandatory_table::mandatory_field);
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query(
    "CREATE TABLE parent_table (
        id INTEGER PRIMARY KEY NOT NULL,
        parent_field TEXT NOT NULL
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE mandatory_table (
        id INTEGER PRIMARY KEY NOT NULL,
        parent_id INTEGER NOT NULL REFERENCES parent_table(id),
        mandatory_field TEXT,
        UNIQUE(id, mandatory_field),
        UNIQUE(id, parent_id)
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE child_table (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
        mandatory_id INTEGER NOT NULL REFERENCES mandatory_table(id),
        child_field TEXT NOT NULL,
        remote_mandatory_field TEXT,
        FOREIGN KEY (mandatory_id, id) REFERENCES mandatory_table(id, parent_id),
        FOREIGN KEY (mandatory_id, remote_mandatory_field) REFERENCES mandatory_table(id, mandatory_field)
    );"
).execute(&mut conn)?;

// Create Child with associated Mandatory (which automatically creates Parent)
let child = child_table::table::builder()
    .parent_field("Parent value")
    .child_field("Child value")
    .mandatory(
        mandatory_table::table::builder()
            .mandatory_field(Some("Mandatory value".to_owned()))
    )
    .insert(&mut conn)?;

// Access the associated Mandatory record
let mandatory: Mandatory = child.mandatory(&mut conn)?;
assert_eq!(mandatory.mandatory_field.as_deref(), Some("Mandatory value"));
let mandatory_parent: Parent = mandatory.parent(&mut conn)?;
// Access the associated Parent record
let parent: Parent = child.ancestor(&mut conn)?;
assert_eq!(parent.parent_field(), "Parent value");
assert_eq!(parent, mandatory_parent);

Ok::<(), Box<dyn std::error::Error>>(())
```

### 6. Discretionary Triangular Relation

[Similar to the mandatory triangular relation](diesel-builders/tests/test_discretionary_triangular_relation.rs), but the constraint is relaxed. Child can reference any Discretionary record, not necessarily one that shares the same Parent. The builder provides `try_discretionary()` for creating new related records or `try_discretionary_model()` for referencing existing ones. Foreign key relationships are declared via `fk!` macro, with composite indices via `index!` macro. Insertion order varies: for builder - Parent → Discretionary → Child; for model - Discretionary exists independently, then Parent → Child references it.

```mermaid
erDiagram
    PARENT_TABLE ||--o{ DISCRETIONARY_TABLE : references
    PARENT_TABLE ||--o| CHILD_WITH_DISCRETIONARY_TABLE : extends
    DISCRETIONARY_TABLE ||--o{ CHILD_WITH_DISCRETIONARY_TABLE : "references"
    PARENT_TABLE {
        int id PK
        string parent_field
    }
    DISCRETIONARY_TABLE {
        int id PK
        int parent_id FK "May differ from Child.id"
        string discretionary_field
    }
    CHILD_WITH_DISCRETIONARY_TABLE {
        int id PK,FK "Inherits from Parent"
        int discretionary_id FK "References any Discretionary"
        string child_field
        string remote_discretionary_field FK
    }
```

```rust
use diesel_builders::prelude::*;
use std::convert::Infallible;

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
    discretionary_field: Option<String>,
}

diesel::allow_tables_to_appear_in_same_query!(parent_table, discretionary_table);
fpk!(discretionary_table::parent_id -> parent_table);
index!(discretionary_table::id, discretionary_table::discretionary_field);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_with_discretionary_table)]
#[table_model(ancestors = parent_table)]
pub struct Child {
    id: i32,
    #[discretionary(discretionary_table)]
    discretionary_id: i32,
    child_field: String,
    remote_discretionary_field: Option<String>,
}

fk!((child_with_discretionary_table::discretionary_id, child_with_discretionary_table::remote_discretionary_field) 
    -> (discretionary_table::id, discretionary_table::discretionary_field));

impl diesel_builders::HorizontalKey for child_with_discretionary_table::discretionary_id {
    type HostColumns = (
        child_with_discretionary_table::id,
        child_with_discretionary_table::remote_discretionary_field,
    );
    type ForeignColumns = (
        discretionary_table::parent_id,
        discretionary_table::discretionary_field,
    );
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query(
    "CREATE TABLE parent_table (
        id INTEGER PRIMARY KEY NOT NULL,
        parent_field TEXT NOT NULL
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE discretionary_table (
        id INTEGER PRIMARY KEY NOT NULL,
        parent_id INTEGER NOT NULL REFERENCES parent_table(id),
        discretionary_field TEXT,
        UNIQUE(id, discretionary_field)
    );"
).execute(&mut conn)?;

diesel::sql_query(
    "CREATE TABLE child_with_discretionary_table (
        id INTEGER PRIMARY KEY NOT NULL REFERENCES parent_table(id),
        discretionary_id INTEGER NOT NULL REFERENCES discretionary_table(id),
        child_field TEXT NOT NULL,
        remote_discretionary_field TEXT,
        FOREIGN KEY (discretionary_id, remote_discretionary_field) 
            REFERENCES discretionary_table(id, discretionary_field)
    );"
).execute(&mut conn)?;

// Example 1: Using a builder to create a new Discretionary record
let child = child_with_discretionary_table::table::builder()
    .parent_field("Parent value")
    .child_field("Child value")
    .discretionary(
        discretionary_table::table::builder()
            .discretionary_field(Some("New discretionary".to_owned()))
    )
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

### 7. Validation with Check Constraints

[Custom validation rules](diesel-builders/tests/test_inheritance.rs) can be implemented through the `ValidateColumn` trait to mirror [SQL CHECK CONSTRAINT](https://www.postgresql.org/docs/current/ddl-constraints.html), providing fail-fast validation before database insertion. This example demonstrates runtime validation, compile-time validation of default values using the `const_validator` macro, and proper error handling.

```rust
use diesel_builders::prelude::*;
use std::convert::Infallible;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = users)]
#[table_model(error = UserError, surrogate_key)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    #[table_model(default = 18)]
    age: i32,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum UserError {
    #[error("Username cannot be empty")]
    UsernameEmpty,
    #[error("Username cannot exceed 50 characters")]
    UsernameTooLong,
    #[error("Email must contain @ symbol")]
    InvalidEmail,
    #[error("Age must be at least 18")]
    AgeTooYoung,
}

impl From<Infallible> for UserError {
    fn from(inf: Infallible) -> Self { match inf {} }
}

// Runtime validation for username
impl ValidateColumn<users::username> for <users::table as TableExt>::NewValues {
    type Error = UserError;
    
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            return Err(UserError::UsernameEmpty);
        }
        if value.len() > 50 {
            return Err(UserError::UsernameTooLong);
        }
        Ok(())
    }
}

// Runtime validation for email
impl ValidateColumn<users::email> for <users::table as TableExt>::NewValues {
    type Error = UserError;
    
    fn validate_column(value: &String) -> Result<(), Self::Error> {
        if !value.contains('@') {
            return Err(UserError::InvalidEmail);
        }
        Ok(())
    }
}

// Compile-time validation for age (including default value)
#[diesel_builders_macros::const_validator]
impl ValidateColumn<users::age> for <users::table as TableExt>::NewValues {
    type Error = UserError;
    
    fn validate_column(value: &i32) -> Result<(), Self::Error> {
        if *value < 18 {
            return Err(UserError::AgeTooYoung);
        }
        Ok(())
    }
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query(
    "CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        username TEXT NOT NULL CHECK (username <> '' AND length(username) <= 50),
        email TEXT NOT NULL CHECK (email LIKE '%@%'),
        age INTEGER NOT NULL CHECK (age >= 18)
    );"
).execute(&mut conn)?;

// Valid insertion using default age
let user = users::table::builder()
    .try_username("alice")?
    .try_email("alice@example.com")?
    .insert(&mut conn)?;

assert_eq!(user.username(), "alice");
assert_eq!(*user.age(), 18); // Default value was validated at compile time

// Valid insertion with explicit age
let user2 = users::table::builder()
    .try_username("bob")?
    .try_email("bob@example.com")?
    .try_age(25)?
    .insert(&mut conn)?;

assert_eq!(*user2.age(), 25);

// Runtime validation errors
let result = users::table::builder()
    .try_username("");  // Error: UsernameEmpty

assert!(result.is_err());
assert_eq!(result.unwrap_err(), UserError::UsernameEmpty);

let result = users::table::builder()
    .try_username("valid_user")?
    .try_email("invalid-email");  // Error: InvalidEmail

assert!(result.is_err());
assert_eq!(result.unwrap_err(), UserError::InvalidEmail);

// Compile-time validated default prevents this at compile time:
// #[table_model(default = 16)]  // Would fail to compile!
// age: i32,

Ok::<(), Box<dyn std::error::Error>>(())
```

### 8. Custom Types

[Custom Diesel types](diesel-builders/tests/test_custom_type.rs) are fully supported. The builder pattern works seamlessly with custom SQL and Rust types that implement the required Diesel traits (`AsExpression`, `FromSql`, `ToSql`).

### 9. Composite Primary Keys

[Tables with multi-column primary keys](examples/composite_primary_keys.rs) are fully supported. The builder pattern works seamlessly with composite keys, allowing type-safe construction and insertion.

```rust
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = user_roles)]
#[diesel(primary_key(user_id, role_id))]
pub struct UserRole {
    user_id: i32,
    role_id: i32,
    #[table_model(default = "2025-01-01")]
    assigned_at: String,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query(
    "CREATE TABLE user_roles (
        user_id INTEGER NOT NULL,
        role_id INTEGER NOT NULL,
        assigned_at TEXT NOT NULL,
        PRIMARY KEY (user_id, role_id)
    );"
).execute(&mut conn)?;

let user_role = user_roles::table::builder()
    .user_id(1)
    .role_id(42)
    .assigned_at("2025-12-12")
    .insert(&mut conn)?;

Ok::<(), Box<dyn std::error::Error>>(())
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
