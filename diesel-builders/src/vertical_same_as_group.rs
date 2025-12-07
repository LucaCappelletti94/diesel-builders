//! Column which is associated to a group of vertical same-as columns.

use crate::{AncestorOfIndex, Columns, DescendantOf, TypedColumn};

/// A trait for Diesel columns that are associated with a group of vertical
/// same-as columns.
pub trait VerticalSameAsGroup<T: DescendantOf<Self::Table>>:
    TypedColumn<Table: AncestorOfIndex<T>>
{
    /// The group of vertical same-as columns associated with this column.
    type VerticalSameAsColumns: Columns;
}
