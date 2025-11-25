//! Submodule providing the `SetBuilder` trait.

use crate::{BuildableColumn, TableBuilder};

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait SetBuilder<Column: BuildableColumn> {
    /// Attempt to set the value of the specified column.
    fn set(&mut self, builder: TableBuilder<<Column as diesel::Column>::Table>);
}

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetMandatoryBuilder<Column: BuildableColumn> {
    /// Attempt to set the value of the specified column.
    fn try_set(
        &mut self,
        builder: TableBuilder<<Column as diesel::Column>::Table>,
    ) -> anyhow::Result<()>;
}
