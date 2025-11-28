//! Column which is associated to a group of vertical same-as columns.

use diesel_additions::{HomogeneousColumns, TypedColumn};

use crate::{AncestorOfIndex, DescendantOf};

/// A trait for Diesel columns that are associated with a group of vertical
/// same-as columns.
pub trait VerticalSameAsGroup<T: DescendantOf<Self::Table>>:
    TypedColumn<Table: AncestorOfIndex<T>>
{
    /// The group of vertical same-as columns associated with this column.
    type VerticalSameAsColumns: HomogeneousColumns<<Self as TypedColumn>::Type>;
}

impl<C> VerticalSameAsGroup<C::Table> for C
where
    C: TypedColumn,
    C::Table: AncestorOfIndex<C::Table>,
{
    type VerticalSameAsColumns = ();
}
