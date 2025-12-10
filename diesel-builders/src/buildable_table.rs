//! Submodule providing the `BuildableTables` trait and its implementations.

use tuplities::prelude::NestedTupleTryFrom;

use crate::{
    BundlableTable, IncompleteBuilderError, NestedBundlableTables, TableBuilder,
    ancestors::DescendantWithSelf, builder_bundle::BundlableTableExt,
};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable: BundlableTableExt + DescendantWithSelf {
    /// The ancestor builders associated with this table.
    type NestedAncestorBuilders: Default;
    /// The completed ancestor builders associated with this table.
    type NestedCompletedAncestorBuilders: NestedTupleTryFrom<Self::NestedAncestorBuilders, IncompleteBuilderError>;

    /// Returns a new instance of a builder for the current table.
    #[inline]
    #[must_use]
    fn builder() -> TableBuilder<Self> {
        TableBuilder::default()
    }
}

impl<T> BuildableTable for T
where
    T: BundlableTable + DescendantWithSelf,
{
    type NestedAncestorBuilders =
        <T::NestedAncestorsWithSelf as NestedBundlableTables>::NestedBundleBuilders;
    type NestedCompletedAncestorBuilders =
        <T::NestedAncestorsWithSelf as NestedBundlableTables>::NestedCompletedBundleBuilders;
}
