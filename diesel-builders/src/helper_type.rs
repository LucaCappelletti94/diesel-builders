//! Submodule defining helper types for the crate.

use crate::{DescendantWithSelf, NestedTables};

/// Helper type alias for the nested model of a table which is a descendant of
/// itself.
///
/// This type alias resolves to the nested models of the table's self-inclusive
/// ancestors.
pub type NestedModel<T> =
    <<T as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels;
