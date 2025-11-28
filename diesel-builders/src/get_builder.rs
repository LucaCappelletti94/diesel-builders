//! Submodule providing the `GetBuilder` trait.

use crate::{BuildableColumn, TableBuilder};

/// Trait providing a getter for a specific Diesel column's builder.
pub trait GetBuilder<Column: BuildableColumn> {
    /// Get the builder for the specified column's table.
    fn get(&self) -> &TableBuilder<<Column as diesel::Column>::Table>;
}

/// Trait providing a failable getter for a specific Diesel column's builder.
pub trait MayGetBuilder<Column: BuildableColumn> {
    /// Get the builder for the specified column's table, returning `None` if
    /// not present.
    fn may_get(&self) -> Option<&TableBuilder<<Column as diesel::Column>::Table>>;
}
