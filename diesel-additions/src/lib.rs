#![doc = include_str!("../README.md")]

#[macro_use]
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
pub use get_column::{GetColumn, MayGetColumn};
pub mod get_set_columns;
pub use get_set_columns::{
    GetColumns, MayGetColumns, SetColumns, SetHomogeneousColumn, TryMaySetColumns, TrySetColumns,
    TrySetHomogeneousColumn,
};
pub mod columns;
pub use columns::{Columns, HomogeneousColumns, Projection};
pub mod table_addition;
pub use table_addition::{HasTableAddition, TableAddition};
pub mod set_column;
pub use set_column::{SetColumn, TrySetColumn};
pub mod insertable_table_model;
pub use insertable_table_model::{
    InsertableTableModel, MayGetInsertableTableModelColumn, SetInsertableTableModelColumn,
    TrySetInsertableTableModelColumn,
};
pub mod foreign_key;
pub use foreign_key::{ForeignKey, SingleColumnForeignKey, SingletonForeignKey};
pub mod insert;
pub use insert::Insert;
pub mod table_inherits;
pub use table_inherits::TableInherits;
