//! Submodule providing the `BuildableTables` trait and its implementations.

use diesel_relations::ancestors::DescendantWithSelf;

use crate::buildable_columns::BuildableColumns;

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
pub trait BuildableTable: DescendantWithSelf {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularSameAsColumns: BuildableColumns;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularSameAsColumns: BuildableColumns;
}
