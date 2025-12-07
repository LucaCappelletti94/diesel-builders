//! Module for buildable columns in Diesel.

use crate::{BuildableTable, TypedColumn};

/// A trait for Diesel columns that can be built.
pub trait BuildableColumn: TypedColumn<Table: BuildableTable> {}

impl<C> BuildableColumn for C
where
    C: TypedColumn,
    C::Table: BuildableTable,
{
}
