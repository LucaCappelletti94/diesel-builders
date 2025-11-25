#![doc = include_str!("../README.md")]

#[macro_use]
extern crate diesel_additions;

pub mod buildable_columns;
pub mod buildable_table;
pub mod buildable_tables;
pub mod table_builder;
pub use buildable_columns::{BuildableColumn, BuildableColumns};
pub use buildable_table::BuildableTable;
pub use buildable_tables::BuildableTables;
pub use table_builder::TableBuilder;
pub mod set_builder;
pub use set_builder::{SetBuilder, TrySetBuilder};
pub mod get_builder;
pub use get_builder::{GetBuilder, MayGetBuilder};
pub mod nested_insert;
pub use nested_insert::NestedInsert;
pub mod builder_bundle;
pub use builder_bundle::TableBuilderBundle;