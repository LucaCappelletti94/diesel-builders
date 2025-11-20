//! Submodule providing the `GetBuilder` trait.

mod for_tuple;

use crate::{BuildableColumn, TableBuilder};

/// Trait providing a failable getter for a specific Diesel column's builder.
pub trait MayGetBuilder<Column: BuildableColumn> {
    /// Get the builder for the specified column's table, returning `None` if
    /// not present.
    fn maybe_get(&self) -> Option<&TableBuilder<<Column as diesel::Column>::Table>>;
}
