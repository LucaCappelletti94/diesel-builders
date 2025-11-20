//! Submodule providing the `SetColumn` trait.

use crate::TypedColumn;

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn> {
    /// Set the value of the specified column.
    fn set(&mut self, value: &<Column as TypedColumn>::Type);
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetColumn<Column: TypedColumn> {
    /// Attempt to set the value of the specified column.
    fn try_set(&mut self, value: &<Column as TypedColumn>::Type) -> anyhow::Result<()>;
}
