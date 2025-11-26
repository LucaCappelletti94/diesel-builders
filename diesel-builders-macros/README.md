# diesel-builders-macros

Procedural macros for the diesel-builders workspace.

This crate provides derive macros and attribute macros for generating trait implementations
on tuples with cleaner syntax than declarative macros.

## Macros

- `#[derive(Columns)]` - Implements `Columns`, `Projection`, and `HomogeneousColumns` traits
- `#[derive(Tables)]` - Implements `Tables`, `TableModels`, and `InsertableTableModels` traits
- `#[derive(BuildableColumns)]` - Implements `BuildableColumns` trait
- `#[derive(BuildableTables)]` - Implements `BuildableTables` trait
- `#[derive(HorizontalSameAsKeys)]` - Implements `HorizontalSameAsKeys` trait
- `#[derive(DefaultTuple)]` - Implements `DefaultTuple` trait
- `#[derive(OptionTuple)]` - Implements `OptionTuple` and `TransposeOptionTuple` traits
- `#[derive(GetColumns)]` - Implements `GetColumns`, `MayGetColumns`, `SetColumns`, and related traits
- `#[derive(NestedInsertTuple)]` - Implements `NestedInsertTuple` trait
- `#[derive(NestedInsertOptionTuple)]` - Implements `NestedInsertOptionTuple` trait
