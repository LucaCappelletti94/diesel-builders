# Diesel Builders

[![Documentation](https://docs.rs/diesel-builders/badge.svg)](https://docs.rs/diesel-builders)
[![CI](https://github.com/LucaCappelletti94/diesel-builders/workflows/Rust%20CI/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Security Audit](https://github.com/LucaCappelletti94/diesel-builders/workflows/Security%20Audit/badge.svg)](https://github.com/LucaCappelletti94/diesel-builders/actions)
[![Codecov](https://codecov.io/gh/LucaCappelletti94/diesel-builders/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/diesel-builders.svg)](https://crates.io/crates/diesel-builders)

A type-safe builder pattern library for [Diesel](https://diesel.rs) that handles complex table relationships including arbitrary inheritance (including DAG dependencies), foreign keys, and both mandatory and optional triangular dependencies. Diesel Builders provides compile-time guarantees for proper insertion order and referential integrity in databases with complex schemas.

It additionally offers ergonomic APIs for getting/setting column values and associated builders and models.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
diesel-builders = "0.1"
```

## Supported Patterns

### 1. Simple Table (Base Case)

A single table with no relationships. This demonstrates the most basic usage of the builder pattern with type-safe column setters and optional validation through `TrySetColumn` trait implementations.

```mermaid
classDiagram
    class Users {
        +Integer id PK
        +Text name
        +Text email
        +Text bio
    }
```

See the [simple_table.rs example](examples/simple_table.rs) for a complete working implementation including SQL CHECK constraints.

### 2. Table Inheritance

Tables extending a parent table via foreign key on the primary key. When inserting into a child table, the builder automatically creates the parent record and ensures proper referential integrity. The `#[descendant_of]` macro declares the inheritance relationship.

```mermaid
classDiagram
    direction BT
    class Users {
        +Integer id PK
        +Text name
        +Text email
    }
    class UserProfiles {
        +Integer id PK,FK
        +Text bio
        +Text avatar_url
    }
    UserProfiles --|> Users : extends
```

See the [table_inheritance.rs example](examples/table_inheritance.rs) for a complete working implementation.

### 3. Directed Acyclic Graph (DAG)

Multiple inheritance where a child table extends multiple parent tables. Table D extends both B and C, which both extend A. The builder automatically resolves the dependency graph and inserts records in the correct order (A → B, C → D), ensuring all foreign key constraints are satisfied.

```mermaid
classDiagram
    direction BT
    class TableA {
        +Integer id PK
        +Text column_a
    }
    class TableB {
        +Integer id PK,FK
        +Text column_b
    }
    class TableC {
        +Integer id PK,FK
        +Text column_c
    }
    class TableD {
        +Integer id PK,FK
        +Text column_d
    }
    TableB --|> TableA : extends
    TableC --|> TableA : extends
    TableD --|> TableB : extends
    TableD --|> TableC : extends
```

See the [dag.rs example](examples/dag.rs) for a complete working implementation.

### 4. Inheritance Chain

A linear inheritance chain where each table extends exactly one parent (A → B → C). The builder automatically determines and enforces the correct insertion order through the dependency graph.

```mermaid
classDiagram
    direction BT
    class TableA {
        +Integer id PK
        +Text column_a
    }
    class TableB {
        +Integer id PK,FK
        +Text column_b
    }
    class TableC {
        +Integer id PK,FK
        +Text column_c
    }
    TableB --|> TableA : extends
    TableC --|> TableB : extends
```

See the [inheritance_chain.rs example](examples/inheritance_chain.rs) for a complete working implementation.

### 5. Mandatory Triangular Relation

A complex pattern where Table B extends A and also references Table C, with the constraint that the C record must also reference the same A record (enforcing `B.c_id == C.a_id == A.id`). The builder uses `set_mandatory_builder` to create both B and its related C record atomically, ensuring referential consistency.

```mermaid
classDiagram
    direction BT
    class TableA {
        +Integer id PK
        +Text column_a
    }
    class TableC {
        +Integer id PK
        +Integer a_id FK
        +Text column_c
    }
    class TableB {
        +Integer id PK,FK
        +Integer c_id FK
        +Text column_b
        +Text remote_column_c
    }
    TableC --> TableA : references
    TableB --|> TableA : extends
    TableB --> TableC : c_id→id
    note for TableB "c_id must reference C where C.a_id = B.id"
```

See the test suite (`tests/test_mandatory_triangular_relation.rs`) for a complete working implementation.

### 6. Discretionary Triangular Relation

Similar to the mandatory triangular relation, but the constraint is relaxed—Table B can reference any C record, not necessarily one that shares the same A parent. The builder provides `set_discretionary_builder` for creating new related records or `set_discretionary_model` for referencing existing ones.

```mermaid
classDiagram
    direction BT
    class TableA {
        +Integer id PK
        +Text column_a
    }
    class TableC {
        +Integer id PK
        +Integer a_id FK
        +Text column_c
    }
    class TableB {
        +Integer id PK,FK
        +Integer c_id FK
        +Text column_b
        +Text remote_column_c
    }
    TableC --> TableA : references
    TableB --|> TableA : extends
    TableB --> TableC : c_id→id
    note for TableB "c_id may reference any C (not required to match B.id)"
```

See the test suite at [test_discretionary_triangular_relation.rs](diesel-builders/tests/test_discretionary_triangular_relation.rs) for complete working examples.

### 7. Composite Primary Keys

Tables with multi-column primary keys are fully supported. The builder pattern works seamlessly with composite keys, allowing type-safe construction and insertion.

```mermaid
classDiagram
    class UserRoles {
        +Integer user_id PK
        +Integer role_id PK
        +Text assigned_at
    }
```

See the [composite_primary_keys.rs example](examples/composite_primary_keys.rs) for a complete working example.

## Macro Attributes

- `#[derive(Root)]`: Marks a table as a root (no parent tables)
- `#[derive(TableModel)]`: Generates model-to-table associations
- `#[derive(GetColumn, SetColumn)]`: Generates type-safe column accessors
- `#[descendant_of]`: Declares parent table relationships
- `#[bundlable_table]`: Configures triangular relationship columns

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
