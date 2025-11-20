//! Submodule defining the `Key` trait for objects strictly associated with a
//! Diesel table.

/// A trait for objects that are strictly associated with a Diesel table.
pub trait Key {
    /// The Diesel table associated with the object.
    type Table: diesel::Table;
}

// Recursive macro that implements `Columns` for tuples of decreasing length.
// Call it with a list of type idents and it will generate impls for the full
// tuple, then the tuple without the first element, and so on, down to 1.
macro_rules! impl_key {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> Key for ($head,)
		where $head: diesel::Column
		{
			type Table = $head::Table;
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> Key for ($head, $($tail),+)
		where $head: diesel::Column, $($tail: diesel::Column<Table = $head::Table>),+
		{
			type Table = $head::Table;
		}

		impl_key!($($tail),+);
	};
}

// Generate implementations for tuples up to 32 elements.
impl_key!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
