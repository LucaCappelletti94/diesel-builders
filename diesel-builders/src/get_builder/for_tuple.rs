//! Submodule implementing the `GetBuilder` and `MayGetBuilder` traits for
//! tuples.

use tuple_set::TupleSet;

use crate::{BuildableTable, MayGetBuilder, TableBuilder};

// The case for 1-element tuples is trivial.
impl<C> MayGetBuilder<C> for (TableBuilder<<C as diesel::Column>::Table>,)
where
    C: crate::BuildableColumn,
    <C as diesel::Column>::Table: BuildableTable,
{
    fn maybe_get(&self) -> Option<&TableBuilder<<C as diesel::Column>::Table>> {
        Some(&self.0)
    }
}

macro_rules! impl_get_builder {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<C, $head, $($tail),+> MayGetBuilder<C> for ($head, $($tail),+)
		where
			C: crate::BuildableColumn,
			<C as diesel::Column>::Table: BuildableTable,
			TableBuilder<<C as diesel::Column>::Table>: 'static,
			Self: TupleSet
		{
			fn maybe_get(&self) -> Option<&TableBuilder<<C as diesel::Column>::Table>> {
				<Self as TupleSet>::get::<TableBuilder<<C as diesel::Column>::Table>>(self)
			}
		}

		impl_get_builder!($($tail),+);
	};
}

generate_tuple_impls!(impl_get_builder);
