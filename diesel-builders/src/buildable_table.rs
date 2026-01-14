//! Submodule providing the `BuildableTables` trait and its implementations.

use tuplities::prelude::{NestedTupleIndex, NestedTupleTryFrom};

use crate::{
    AncestorOfIndex, IncompleteBuilderError, TableBuilder, TableBuilderBundle,
    ancestors::DescendantWithSelf, builder_bundle::BundlableTableExt,
};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
///
/// This trait provides the core functionality for creating builders that handle
/// complex table relationships including inheritance hierarchies and triangular
/// dependencies. It ensures that all required related records are created in
/// the correct order.
///
/// Extends [`BundlableTableExt`] and [`DescendantWithSelf`].
///
/// # Type Parameters
///
/// * `NestedAncestorBuilders`: A nested tuple of builder bundles for ancestor
///   tables
/// * `NestedCompletedAncestorBuilders`: The completed version of ancestor
///   builders ready for insertion
pub trait BuildableTable: BundlableTableExt + DescendantWithSelf {
    /// The ancestor builders associated with this table.
    type NestedAncestorBuilders: Default
        + NestedTupleIndex<<Self as AncestorOfIndex<Self>>::Idx, Element = TableBuilderBundle<Self>>;
    /// The completed ancestor builders associated with this table.
    type NestedCompletedAncestorBuilders: NestedTupleTryFrom<Self::NestedAncestorBuilders, IncompleteBuilderError>;

    #[must_use]
    /// Returns the default ancestor builders for this table.
    ///
    /// This provides empty builder bundles for all ancestor tables in the
    /// inheritance hierarchy.
    fn default_bundles() -> Self::NestedAncestorBuilders;

    /// Returns a new instance of a builder for the current table.
    ///
    /// This is the primary entry point for creating records with complex
    /// relationships. The builder will handle dependency ordering
    /// automatically.
    #[inline]
    #[must_use]
    fn builder() -> TableBuilder<Self> {
        TableBuilder { bundles: Self::default_bundles() }
    }
}
