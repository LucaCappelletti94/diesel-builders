//! Submodule providing the `SetBuilder` trait.

mod for_tuple;

use crate::{BuildableColumn, TableBuilder};

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait TrySetBuilder<Column: BuildableColumn> {
    /// Attempt to set the value of the specified column.
    fn try_set(
        &mut self,
        builder: TableBuilder<<Column as diesel::Column>::Table>,
    ) -> anyhow::Result<()>;
}
