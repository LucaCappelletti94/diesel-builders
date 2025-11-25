//! Submodule defining and implementing the `Columns` trait.

use crate::TypedColumn;

/// A trait representing a collection of Diesel columns.
pub trait Columns {}

/// A trait representing a projection of Diesel columns.
pub trait Projection {
    /// The table to which these columns belong.
    type Table: diesel::Table;
}

/// A trait representing a collection of Diesel columns with an associated type.
pub trait HomogeneousColumns: Columns {
    /// The associated tuple type of the columns.
    type Type;
}

/// A trait representing columns that are horizontally same-as (same type across different tables).
pub trait HorizontalSameAsColumns: HomogeneousColumns {}

impl Columns for () {}

// Recursive macro that implements `Columns` for tuples of decreasing length.
// Call it with a list of type idents and it will generate impls for the full
// tuple, then the tuple without the first element, and so on, down to 1.
macro_rules! impl_columns {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> Columns for ($head,)
		where $head: diesel::Column
		{
		}
		impl<$head> Projection for ($head,)
		where $head: diesel::Column
		{
			type Table = <$head as diesel::Column>::Table;
		}
		impl<$head> HomogeneousColumns for ($head,)
		where $head: TypedColumn
		{
			type Type = <$head as TypedColumn>::Type;
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> Columns for ($head, $($tail),+)
		where $head: diesel::Column, $($tail: diesel::Column),+
		{
		}
		impl<$head, $($tail),+> Projection for ($head, $($tail),+)
		where $head: diesel::Column, $($tail: diesel::Column<Table=<$head as diesel::Column>::Table>),+
		{
			type Table = <$head as diesel::Column>::Table;
		}
		impl<$head, $($tail),+> HomogeneousColumns for ($head, $($tail),+)
		where $head: TypedColumn, $($tail: TypedColumn),+
		{
			type Type = <$head as TypedColumn>::Type;
		}

		impl_columns!($($tail),+);
	};
}

generate_tuple_impls!(impl_columns);
