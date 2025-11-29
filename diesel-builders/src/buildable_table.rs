//! Submodule providing the `BuildableTables` trait and its implementations.

use diesel::Table;
use diesel_additions::{GetColumn, table_addition::HasPrimaryKey};
use diesel_relations::{
    AncestorOfIndex, DescendantOf, ancestors::DescendantWithSelf,
    vertical_same_as_group::VerticalSameAsGroup,
};

use crate::{BundlableTable, BundlableTables, TableBuilder};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable:
    BundlableTable + DescendantWithSelf<AncestorsWithSelf: BundlableTables>
{
    /// Returns a new instance of a builder for the current table.
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
