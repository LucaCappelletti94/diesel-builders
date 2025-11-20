//! Submodule providing the `GetColumn` trait.

mod for_tuple;

use crate::TypedColumn;

/// Trait providing a getter for a specific Diesel column.
pub trait GetColumn<Column: TypedColumn> {
    /// Get the value of the specified column.
    fn get(&self) -> &<Column as TypedColumn>::Type;
}

/// Trait providing a failable getter for a specific Diesel column.
pub trait MayGetColumn<Column: TypedColumn> {
    /// Get the value of the specified column, returning `None` if not present.
    fn maybe_get(&self) -> Option<&<Column as TypedColumn>::Type>;
}
