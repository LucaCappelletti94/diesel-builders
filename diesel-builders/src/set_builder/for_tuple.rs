//! Submodule implementing the `TrySetBuilder` trait for tuples.

use tuple_set::TupleSet;

use crate::{BuildableTable, TableBuilder, TrySetBuilder};

// The case for 1-element tuples is trivial.
impl<C> TrySetBuilder<C> for (TableBuilder<<C as diesel::Column>::Table>,)
where
    C: crate::BuildableColumn,
    <C as diesel::Column>::Table: BuildableTable,
{
    fn try_set(
        &mut self,
        builder: TableBuilder<<C as diesel::Column>::Table>,
    ) -> anyhow::Result<()> {
        self.0 = builder;
        Ok(())
    }
}

macro_rules! impl_set_builder {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<C, $head, $($tail),+> TrySetBuilder<C> for ($head, $($tail),+)
		where
			C: crate::BuildableColumn,
			<C as diesel::Column>::Table: BuildableTable,
			TableBuilder<<C as diesel::Column>::Table>: 'static,
			Self: TupleSet
		{
			fn try_set(&mut self, builder: TableBuilder<<C as diesel::Column>::Table>) -> anyhow::Result<()> {
				self.map(|elem: &mut TableBuilder<<C as diesel::Column>::Table>| {
					*elem = builder;
					Ok(())
				}).unwrap_or_else(|| anyhow::bail!(
					"Type {table_builder} was not found in tuple {} for column {}.",
					std::any::type_name::<Self>(),
					std::any::type_name::<C>(),
					table_builder = std::any::type_name::<TableBuilder<<C as diesel::Column>::Table>>(),
				))
			}
		}

		impl_set_builder!($($tail),+);
	};
}

generate_tuple_impls!(impl_set_builder);
