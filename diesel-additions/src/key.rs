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

generate_tuple_impls!(impl_key);
