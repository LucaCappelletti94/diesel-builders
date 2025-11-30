#![doc = include_str!("../README.md")]

// Re-exported modules from diesel-additions
pub mod utils;
pub use utils::*;
pub mod tables;
pub use tables::{NonCompositePrimaryKeyTables, Tables};
pub mod table_model;
pub use table_model::{
    NonCompositePrimaryKeyTableModel, NonCompositePrimaryKeyTableModels, TableModel,
};
pub mod typed_column;
pub use typed_column::TypedColumn;
pub mod get_column;
pub use get_column::{GetColumn, GetColumnExt, MayGetColumn, MayGetColumnExt};
pub mod get_set_columns;
pub use get_set_columns::{
    GetColumns, MayGetColumns, MaySetColumns, SetColumns, SetHomogeneousColumn, TryMaySetColumns,
    TrySetColumns, TrySetHomogeneousColumn,
};
pub mod columns;
pub use columns::{Columns, HomogeneousColumns, Projection};
pub mod table_addition;
pub use table_addition::{HasPrimaryKey, HasTableAddition, TableAddition};
pub mod set_column;
pub use set_column::{
    MaySetColumn, MaySetColumnExt, SetColumn, SetColumnExt, TrySetColumn, TrySetColumnExt,
};
pub mod insertable_table_model;
pub use insertable_table_model::{InsertableTableModel, SetInsertableTableModelColumn};
pub mod foreign_key;
pub use foreign_key::{ForeignKey, SingleColumnForeignKey, SingletonForeignKey};
pub mod flat_insert;
pub use flat_insert::FlatInsert;
pub mod table_inherits;
pub use table_inherits::TableInherits;

// Re-exported modules from diesel-relations
pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as;
pub mod vertical_same_as_group;
pub use ancestors::{AncestorOfIndex, Descendant, DescendantOf, Root};
pub use horizontal_same_as::{
    DiscretionarySameAsIndex, HorizontalSameAsColumn, HorizontalSameAsKey, HorizontalSameAsKeys,
    MandatorySameAsIndex,
};
pub mod horizontal_same_as_group;
pub use horizontal_same_as_group::HorizontalSameAsGroup;

// Original diesel-builders modules
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
    SetDiscretionaryBuilder, SetDiscretionaryBuilderExt, SetDiscretionaryModel,
    SetDiscretionaryModelExt, SetMandatoryBuilder, SetMandatoryBuilderExt,
    TryMaySetDiscretionarySameAsColumn, TryMaySetDiscretionarySameAsColumns,
    TrySetDiscretionaryBuilder, TrySetDiscretionaryBuilderExt, TrySetDiscretionaryModel,
    TrySetDiscretionaryModelExt, TrySetMandatoryBuilder, TrySetMandatoryBuilderExt,
    TrySetMandatorySameAsColumn, TrySetMandatorySameAsColumns,
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
