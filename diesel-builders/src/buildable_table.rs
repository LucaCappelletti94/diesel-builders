//! Submodule providing the `BuildableTables` trait and its implementations.

use diesel_relations::ancestors::DescendantWithSelf;

use crate::{BuildableTables, TableBuilder, TableBundle};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable:
    TableBundle + DescendantWithSelf<AncestorsWithSelf: BuildableTables>
{
    /// Returns a new instance of a builder for the current table.
    fn builder() -> TableBuilder<Self> {
        TableBuilder::default()
    }
}

impl<T> BuildableTable for T where
    T: TableBundle + DescendantWithSelf<AncestorsWithSelf: BuildableTables>
{
}
