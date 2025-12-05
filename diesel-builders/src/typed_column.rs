//! Submodule providing the `TypedColumn` trait.

use std::fmt::Debug;

use crate::TableAddition;

/// Trait representing a Diesel column associated with a specific type.
pub trait TypedColumn: diesel::Column<Table: TableAddition> + Debug + Clone + Default {
    /// The Rust type associated with this column, as it appears in queries.
    type Type: 'static + Clone;
}
