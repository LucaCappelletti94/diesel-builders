//! Submodule providing the `BuildableTables` trait and its implementations.

use diesel::Table;

use crate::{
    AncestorOfIndex, BundlableTable, BundlableTables, DescendantOf, GetColumn, TableBuilder,
    ancestors::DescendantWithSelf, table_addition::HasPrimaryKey,
    vertical_same_as_group::VerticalSameAsGroup,
};

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

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors, where the table has an ancestor-descendant
/// relationship with another table `T`.
pub trait AncestralBuildableTable<T: DescendantOf<Self> + HasPrimaryKey>:
    BuildableTable<
        PrimaryKey: VerticalSameAsGroup<T>,
        Model: GetColumn<<Self as Table>::PrimaryKey>,
    > + AncestorOfIndex<T>
    + HasPrimaryKey
{
}

impl<D, A> AncestralBuildableTable<D> for A
where
    D: DescendantOf<Self> + HasPrimaryKey,
    A: BuildableTable<
            PrimaryKey: VerticalSameAsGroup<D>,
            Model: GetColumn<<A as Table>::PrimaryKey>,
        > + AncestorOfIndex<D>
        + HasPrimaryKey,
{
}
