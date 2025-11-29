#![doc = include_str!("../README.md")]

pub mod buildable_columns;
pub mod buildable_table;
pub mod buildable_tables;
pub mod table_builder;
pub use buildable_columns::{BuildableColumn, BuildableColumns};
pub use buildable_table::{AncestralBuildableTable, BuildableTable};
pub use buildable_tables::BuildableTables;
pub use table_builder::TableBuilder;
pub mod set_builder;
pub use set_builder::{
    SetDiscretionaryBuilder, SetMandatoryBuilder, TrySetDiscretionaryBuilder,
    TrySetMandatoryBuilder,
};
pub mod get_builder;
pub use get_builder::{GetBuilder, MayGetBuilder};
pub mod nested_insert;
pub use nested_insert::NestedInsert;
pub mod builder_bundle;
pub use builder_bundle::{
    BuilderBundles, BundlableTable, CompletedTableBuilderBundle, TableBuilderBundle,
};
pub mod bundlable_tables;
pub use bundlable_tables::BundlableTables;
