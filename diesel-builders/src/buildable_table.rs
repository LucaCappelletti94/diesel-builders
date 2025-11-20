//! Submodule providing the `BuildableTables` trait and its implementations.

use diesel_additions::tables::InsertableTables;
use diesel_relations::ancestors::DescendantWithSelf;

use crate::buildable_columns::BuildableColumns;

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable:
    DescendantWithSelf<UniqueAncestorsWithSelf = Self::InsertableTables>
{
    /// The collection of insertable tables corresponding to the unique
    /// ancestors of this table.
    type InsertableTables: InsertableTables;
    /// The columns defining triangular same-as.
    type TriangularSameAsColumns: BuildableColumns;
}
