//! Module for buildable columns in Diesel.

use diesel_additions::{OptionTuple, Tables};

use crate::{BuildableTable, TableBuilder};

/// A trait for collections of Diesel tables that can be built.
pub trait BuildableTables: Tables {
    /// The builders associated with the buildable tables.
    type Builders: OptionTuple;
}

macro_rules! impl_buildable_tables {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> BuildableTables for ($head,)
		where
			$head: BuildableTable
		{
			type Builders = (TableBuilder<$head>,);
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> BuildableTables for ($head, $($tail),+)
		where
			$head: BuildableTable,
			$($tail: BuildableTable),+
		{
			type Builders = (
				TableBuilder<$head>,
				$(TableBuilder<$tail>),+
			);
		}

		impl_buildable_tables!($($tail),+);
	};
}

// Generate implementations for tuples up to 32 elements.
impl_buildable_tables!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
