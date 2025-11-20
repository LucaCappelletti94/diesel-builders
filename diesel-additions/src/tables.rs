//! Submodule defining and implementing the `Tables` trait.

use crate::InsertableTable;

/// A trait representing a collection of Diesel tables.
pub trait Tables {}

/// A trait representing a collection of Diesel insertable tables.
pub trait InsertableTables {
    /// The insertable models corresponding to the tables in this collection.
    type InsertableModels;
}

impl Tables for () {}

macro_rules! impl_tables {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> Tables for ($head,)
		where
			$head: diesel::Table
		{
		}

		impl<$head> InsertableTables for ($head,)
		where
			$head: InsertableTable
		{
			type InsertableModels = ($head::InsertableModel,);
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> Tables for ($head, $($tail),+)
		where
			$head: diesel::Table,
			$($tail: diesel::Table),+
		{
		}

		impl<$head, $($tail),+> InsertableTables for ($head, $($tail),+)
		where
			$head: InsertableTable,
			$($tail: InsertableTable),+
		{
			type InsertableModels = (
				$head::InsertableModel,
				$($tail::InsertableModel),+
			);
		}

		impl_tables!($($tail),+);
	};
}

generate_tuple_impls!(impl_tables);
