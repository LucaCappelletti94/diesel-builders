//! Submodule providing the `BuildableTables` trait and its implementations.

use tuplities::prelude::{NestedTupleIndex, NestedTupleTryFrom};

use crate::{
    AncestorOfIndex, IncompleteBuilderError, TableBuilder, TableBuilderBundle,
    ancestors::DescendantWithSelf, builder_bundle::BundlableTableExt,
};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable: BundlableTableExt + DescendantWithSelf {
    /// The ancestor builders associated with this table.
    type NestedAncestorBuilders: Default
        + NestedTupleIndex<<Self as AncestorOfIndex<Self>>::Idx, Element = TableBuilderBundle<Self>>;
    /// The completed ancestor builders associated with this table.
    type NestedCompletedAncestorBuilders: NestedTupleTryFrom<Self::NestedAncestorBuilders, IncompleteBuilderError>;

    #[must_use]
    /// Returns the default ancestor builders for this table.
    fn default_bundles() -> Self::NestedAncestorBuilders;

    /// Returns a new instance of a builder for the current table.
    #[inline]
    #[must_use]
    fn builder() -> TableBuilder<Self> {
        TableBuilder {
            bundles: Self::default_bundles(),
        }
    }
}
