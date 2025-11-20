#![doc = include_str!("../README.md")]

pub mod tables;
pub use tables::Tables;
pub mod table_model;
pub use table_model::TableModel;
pub mod typed_column;
pub use typed_column::TypedColumn;
pub mod get_column;
pub use get_column::{GetColumn, MayGetColumn};
pub mod get_set_columns;
pub use get_set_columns::{
    GetColumns, MayGetColumns, SetColumns, TrySetColumns, TrySetHomogeneousColumns,
};
pub mod columns;
pub use columns::{Columns, HomogeneousColumns};
pub mod table_addition;
pub use table_addition::{HasTableAddition, TableAddition};
pub mod set_column;
pub use set_column::{SetColumn, TrySetColumn};
pub mod insertable_table_model;
pub use insertable_table_model::{InsertableTable, InsertableTableModel};
pub mod foreign_key;
pub use foreign_key::ForeignKey;
pub mod key;
pub use key::Key;
pub mod utils;
pub use utils::*;
