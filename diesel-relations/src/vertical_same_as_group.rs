//! Column which is associated to a group of vertical same-as columns.

use diesel_additions::TypedColumn;

use crate::{AncestorOfIndex, DescendantOf};

/// A trait for Diesel columns that are associated with a group of vertical
/// same-as columns.
pub trait VerticalSameAsGroup<T: DescendantOf<Self::Table>>:
    TypedColumn<Table: AncestorOfIndex<T>>
{
    /// The group of vertical same-as columns associated with this column.
    type VerticalSameAsColumns: diesel_additions::HomogeneousColumns<Type = <Self as TypedColumn>::Type>;
}
