//! Submodule providing the `SetBuilder` trait.

use crate::{BuildableColumn, TableBuilder};

/// Trait providing a setter for a specific Diesel column.
pub trait SetBuilder<Column: BuildableColumn> {
    /// Set the value of the specified column.
    fn set(&mut self, builder: TableBuilder<<Column as diesel::Column>::Table>);
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetBuilder<Column: BuildableColumn> {
    /// Attempt to set the value of the specified column.
    fn try_set(
        &mut self,
        builder: TableBuilder<<Column as diesel::Column>::Table>,
    ) -> anyhow::Result<()>;
}
