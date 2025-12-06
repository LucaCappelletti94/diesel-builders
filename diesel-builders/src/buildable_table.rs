//! Submodule providing the `BuildableTables` trait and its implementations.

use crate::{BundlableTable, BundlableTables, TableBuilder, ancestors::DescendantWithSelf};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable:
    BundlableTable + DescendantWithSelf<AncestorsWithSelf: BundlableTables>
{
    /// Returns a new instance of a builder for the current table.
    #[inline]
    #[must_use]
    fn builder() -> TableBuilder<Self> {
        TableBuilder::default()
    }
}

impl<T> BuildableTable for T where
    T: BundlableTable + DescendantWithSelf<AncestorsWithSelf: BundlableTables>
{
}
