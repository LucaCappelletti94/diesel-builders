//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel_additions::TableAddition;
use diesel_relations::HorizontalSameAsKeys;

use crate::BuildableTables;

pub trait TableBundle: TableAddition {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularSameAsColumns: HorizontalSameAsKeys<ReferencedTables: BuildableTables>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<
        ReferencedTables: BuildableTables,
    >;
}

pub struct TableBuilderBundle<T: TableBundle> {
	/// The insertable model for the table.
	insertable_model: T::InsertableModel,
	/// The mandatory associated builders relative to triangular same-as.
	mandatory_associated_builders: <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
}
