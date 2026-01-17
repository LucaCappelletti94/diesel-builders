# Diesel Builders

[![CI](https://github.com/LucaCappelletti94/diesel-builders/workflows/Rust%20CI/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Security Audit](https://github.com/LucaCappelletti94/diesel-builders/workflows/Security%20Audit/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Codecov](https://codecov.io/gh/LucaCappelletti94/diesel-builders/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A type-safe builder pattern library for [Diesel](https://diesel.rs) handling complex table relationships (inheritance chains, DAGs, triangular dependencies) with compile-time guarantees for insertion order and referential integrity. It provides fluent APIs for getting/setting column values and associated builders and models, executing foreign key queries, and [`serde`](https://github.com/serde-rs/serde) support.

[Custom Diesel types](diesel-builders/tests/test_custom_type.rs) and [tables with multi-column primary keys](examples/composite_primary_keys.rs) are fully supported. The builder pattern works seamlessly with custom SQL and Rust types [that implement the required Diesel traits](https://github.com/diesel-rs/diesel/blob/main/guide_drafts/custom_types.md).

The `TableModel` derive macro generates Diesel's [`table!`](https://docs.rs/diesel/latest/diesel/macro.table.html) macro, eliminating manual schema definitions. Furthermore, its also automatically keeps track of foreign key relationships to generate [`allow_tables_to_appear_in_same_query!`](https://docs.rs/diesel/latest/diesel/macro.allow_tables_to_appear_in_same_query.html) declarations as needed. You will still need to specify [`allow_tables_to_appear_in_same_query`](https://docs.rs/diesel/latest/diesel/macro.allow_tables_to_appear_in_same_query.html) for second-order joins (i.e., joins involving three or more tables).

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
// Descendants can override ancestor defaults
#[table_model(default(dogs::dog_notes, "A cute little puppy"))]
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
    .age_months(3)
    .insert(&mut conn)?;

// You can load the table with `find`:
let loaded_puppy: Puppy = Puppy::find(puppy.id(), &mut conn)?;

// Access ancestor records
let animal: Animal = puppy.ancestor(&mut conn)?;
assert_eq!(animal.name(), "Buddy");
assert_eq!(
    animal.description().as_deref(),
    Some("A cute little puppy"),
    "Description should be propagated from Puppy to Animal"
);
let dog: Dog = puppy.ancestor(&mut conn)?;
assert_eq!(dog.breed(), "Labrador");
assert_eq!(
    dog.dog_notes().as_deref(),
    Some("A cute little puppy"),
    "dog_notes should be propagated from Puppy to Dog"
);
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

let pet_graph = pets::table::builder()
    .name("Bellerophon")
    .breed("Hybrid Orange-Labrador")
    .color("Orange")
    .owner_name("Alice Smith")
    // insert_nested returns a nested structure containing all created/referenced models
    .insert_nested(&mut conn)?;

// Directly access fields from any ancestor in the hierarchy
assert_eq!(pet_graph.name(), "Bellerophon");            // From Animal
assert_eq!(pet_graph.breed(), "Hybrid Orange-Labrador"); // From Dog
assert_eq!(pet_graph.color(), "Orange");                 // From Cat
assert_eq!(pet_graph.owner_name(), "Alice Smith");       // From Pet

// The returned structure is a nested tuple: (Animal, (Dog, (Cat, (Pet,))))
// We can access the root model (Animal) to get the ID shared by all tables
let pet_id = pet_graph.get_column::<animals::id>(); 
Pet::find(&pet_id, &mut conn)?.delete(&mut conn)?;
assert!(!Pet::exists(&pet_id, &mut conn)?);
assert!(!Dog::exists(&pet_id, &mut conn)?);
assert!(!Cat::exists(&pet_id, &mut conn)?);
assert!(!Animal::exists(&pet_id, &mut conn)?);

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
#[table_model(foreign_key(parent_id, (parent_table::id)))]
pub struct Mandatory {
    id: i32,
    parent_id: i32,
    #[table_model(default = "Default mandatory")]
    mandatory_field: String,
}

unique_index!(mandatory_table::id, mandatory_table::mandatory_field);
unique_index!(mandatory_table::id, mandatory_table::parent_id);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(ancestors(parent_table))]
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
#[table_model(foreign_key(parent_id, (parent_table::id)))]
pub struct Discretionary {
    id: i32,
    parent_id: i32,
    discretionary_field: String,
}

unique_index!(discretionary_table::id, discretionary_table::discretionary_field);
unique_index!(discretionary_table::id, discretionary_table::parent_id);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_with_discretionary_table)]
#[table_model(ancestors(parent_table))]
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

### 6. Iterating Foreign Keys

[Iterating over foreign keys](diesel-builders/tests/test_iter_foreign_key.rs) grouping by referred index.

```rust
use diesel_builders::prelude::*;
use diesel_builders::DynTypedColumn;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = nodes)]
#[table_model(surrogate_key)]
pub struct Node {
    id: i32,
    name: String,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = edges)]
#[table_model(foreign_key(source_id, (nodes::id)))]
#[table_model(foreign_key(target_id, (nodes::id)))]
pub struct Edge {
    id: i32,
    source_id: i32,
    target_id: i32,
}

let mut conn = SqliteConnection::establish(":memory:")?;

diesel::sql_query("CREATE TABLE nodes (id INTEGER PRIMARY KEY, name TEXT NOT NULL);").execute(&mut conn)?;
diesel::sql_query("CREATE TABLE edges (id INTEGER PRIMARY KEY, source_id INTEGER NOT NULL REFERENCES nodes(id), target_id INTEGER NOT NULL REFERENCES nodes(id));").execute(&mut conn)?;

let node1 = nodes::table::builder().name("Node 1").insert(&mut conn)?;
let node2 = nodes::table::builder().name("Node 2").insert(&mut conn)?;
let node3 = nodes::table::builder().name("Node 3").insert(&mut conn)?;

let edge = edges::table::builder()
    .id(1)
    .source_id(*node1.id())
    .target_id(*node2.id())
    .insert(&mut conn)?;

// Iterate over foreign key values pointing to nodes::id
// The result is a list of references to the foreign key values (nested tuples)
let refs: Vec<_> = edge.iter_match_full::<(nodes::id,)>().collect();

assert_eq!(refs.len(), 2);
assert!(refs.contains(&(&node1.id(),)));
assert!(refs.contains(&(&node2.id(),)));

// Iterate over foreign key columns as dynamic trait objects
// The result is a list of boxed host table columns with value types from the referenced index
let keys: Vec<_> = 
    Edge::iter_foreign_key_columns::<(nodes::id,)>().collect();

assert_eq!(keys.len(), 2);
assert_eq!(keys[0].0.column_name(), edges::source_id.column_name());
assert_eq!(keys[1].0.column_name(), edges::target_id.column_name());

// We can dynamically set the values of the columns using `TrySetDynamicColumn::try_set_dynamic_column`
let mut edge_builder = edges::table::builder().id(2);
for ((column,), (value_ref,)) in keys.iter().zip(refs.iter()) {
    edge_builder.try_set_dynamic_column_ref::<edges::table, i32>(column, **value_ref)?;
}
let edge2 = edge_builder.insert(&mut conn)?;

assert_eq!(edge2.source_id(), edge.source_id());
assert_eq!(edge2.target_id(), edge.target_id());

Ok::<(), Box<dyn std::error::Error>>(())
```

### 7. Validation with Check Constraints

[Custom validation](diesel-builders/tests/test_inheritance.rs) via `ValidateColumn` mirrors SQL CHECK constraints.

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

impl ValidateColumn<users::age> for <users::table as TableExt>::NewValues {
    type Error = UserError;
    
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
assert_eq!(*user.age(), 18);

// Valid insertion with explicit age
let user2 = users::table::builder()
    .username("bob")
    .try_age(25)?
    .insert(&mut conn)?;

assert_eq!(*user2.age(), 25);

// Runtime validation errors
let result = users::table::builder().try_age(7);  // Error: AgeTooYoung
assert_eq!(result.unwrap_err(), UserError::AgeTooYoung);

Ok::<(), Box<dyn std::error::Error>>(())
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
