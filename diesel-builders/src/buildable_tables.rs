//! Module for buildable columns in Diesel.

use diesel_additions::{OptionTuple, Tables};

use crate::{BuildableTable, TableBuilder};

/// A trait for collections of Diesel tables that can be built.
pub trait BuildableTables: Tables {
    /// The builders associated with the buildable tables.
    type Builders: OptionTuple;
}

impl BuildableTables for () {
	type Builders = ();
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

generate_tuple_impls!(impl_buildable_tables);
