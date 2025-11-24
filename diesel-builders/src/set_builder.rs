//! Submodule providing the `SetBuilder` trait.

use crate::{BuildableColumn, TableBuilder};

/// Trait attempting to set a specific Diesel column, which may fail.
pub trait SetBuilder<Column: BuildableColumn> {
    /// Attempt to set the value of the specified column.
    fn set(&mut self, builder: TableBuilder<<Column as diesel::Column>::Table>);
}

/// Trait dispatching `SetBuilder` for Tuples.
pub trait TupleSetBuilder<const INDEX: usize, Column: BuildableColumn> {
    /// Set the builder for the specified column's table.
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

/// Trait dispatching `TrySetBuilder` for Tuples.
pub trait TupleTrySetBuilder<const INDEX: usize, Column: BuildableColumn> {
    /// Attempt to set the builder for the specified column's table.
    fn try_set(
        &mut self,
        builder: TableBuilder<<Column as diesel::Column>::Table>,
    ) -> anyhow::Result<()>;
}
