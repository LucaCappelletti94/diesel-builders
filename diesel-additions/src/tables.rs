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

// Generate implementations for tuples up to 32 elements.
impl_tables!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
