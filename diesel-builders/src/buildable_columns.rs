//! Module for buildable columns in Diesel.

use diesel::Column;
use diesel_additions::Columns;

use crate::{BuildableTable, buildable_tables::BuildableTables};

/// A trait for Diesel columns that can be built.
pub trait BuildableColumn: Column<Table: BuildableTable> {}

impl<C> BuildableColumn for C
where
    C: Column,
    C::Table: BuildableTable,
{
}

/// A trait for collections of Diesel columns that can be built.
pub trait BuildableColumns: Columns {
    /// Associated tables for the buildable columns.
    type Tables: BuildableTables;
}

// Recursive macro that implements `Columns` for tuples of decreasing length.
// Call it with a list of type idents and it will generate impls for the full
// tuple, then the tuple without the first element, and so on, down to 1.
macro_rules! impl_buildable_columns {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> BuildableColumns for ($head,)
		where
			$head: BuildableColumn,
		{
			type Tables = (<$head as diesel::Column>::Table,);
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> BuildableColumns for ($head, $($tail),+)
		where
			$head: BuildableColumn,
			$(
				$tail: BuildableColumn
			),+
		{
			type Tables = (
				<$head as diesel::Column>::Table,
				$(<$tail as diesel::Column>::Table),+
			);
		}

		impl_buildable_columns!($($tail),+);
	};
}

// Generate implementations for tuples up to 32 elements.
impl_buildable_columns!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
