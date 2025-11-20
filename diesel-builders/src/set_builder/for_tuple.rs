//! Submodule implementing the `TrySetBuilder` trait for tuples.

use tuple_set::TupleSet;

use crate::{BuildableTable, SetBuilder, TableBuilder};

// The case for 1-element tuples is trivial.
impl<C> SetBuilder<C> for (TableBuilder<<C as diesel::Column>::Table>,)
where
    C: crate::BuildableColumn,
    <C as diesel::Column>::Table: BuildableTable,
{
    fn set(&mut self, builder: TableBuilder<<C as diesel::Column>::Table>) {
        self.0 = builder;
    }
}

macro_rules! impl_set_builder {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<C, $head, $($tail),+> SetBuilder<C> for ($head, $($tail),+)
		where
			C: crate::BuildableColumn,
			<C as diesel::Column>::Table: BuildableTable,
			TableBuilder<<C as diesel::Column>::Table>: 'static,
			Self: TupleSet
		{
			fn set(&mut self, builder: TableBuilder<<C as diesel::Column>::Table>) {
				self.map(|elem: &mut TableBuilder<<C as diesel::Column>::Table>| {
					*elem = builder;
				});
			}
		}

		impl_set_builder!($($tail),+);
	};
}

generate_tuple_impls!(impl_set_builder);
