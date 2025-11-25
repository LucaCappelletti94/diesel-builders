//! Module for buildable columns in Diesel.

use diesel_additions::{Columns, TypedColumn};

use crate::{BuildableTable, buildable_tables::BuildableTables};

/// A trait for Diesel columns that can be built.
pub trait BuildableColumn: TypedColumn<Table: BuildableTable> {}

impl<C> BuildableColumn for C
where
    C: TypedColumn,
    C::Table: BuildableTable,
{
}

/// A trait for collections of Diesel columns that can be built.
pub trait BuildableColumns: Columns {
    /// Associated tables for the buildable columns.
    type Tables: BuildableTables;
}

impl BuildableColumns for () {
    type Tables = ();
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

generate_tuple_impls!(impl_buildable_columns);
