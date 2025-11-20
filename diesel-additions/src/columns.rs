//! Submodule defining and implementing the `Columns` trait.

use crate::TypedColumn;

/// A trait representing a collection of Diesel columns.
pub trait Columns {}

/// A trait representing a collection of Diesel columns with an associated type.
pub trait HomogeneousColumns: Columns {
    /// The associated tuple type of the columns.
    type Type;
}

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

		impl<$head, $($tail),+> HomogeneousColumns for ($head, $($tail),+)
		where $head: TypedColumn, $($tail: TypedColumn),+
		{
			type Type = <$head as TypedColumn>::Type;
		}

		impl_columns!($($tail),+);
	};
}

// Generate implementations for tuples up to 32 elements.
impl_columns!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
