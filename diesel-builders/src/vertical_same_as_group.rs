//! Column which is associated to a group of vertical same-as columns.

use crate::{HomogeneousColumns, TypedColumn, table_addition::HasPrimaryKey};

use crate::{AncestorOfIndex, DescendantOf};

/// A trait for Diesel columns that are associated with a group of vertical
/// same-as columns.
pub trait VerticalSameAsGroup<T: DescendantOf<Self::Table> + HasPrimaryKey>:
    TypedColumn<Table: AncestorOfIndex<T> + HasPrimaryKey>
{
    /// The group of vertical same-as columns associated with this column.
    type VerticalSameAsColumns: HomogeneousColumns<<Self as TypedColumn>::Type>;
}

impl<C> VerticalSameAsGroup<C::Table> for C
where
    C: TypedColumn<Table: AncestorOfIndex<C::Table> + HasPrimaryKey + DescendantOf<C::Table>>,
{
    type VerticalSameAsColumns = ();
}
