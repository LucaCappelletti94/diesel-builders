//! Submodule implementing the `SetColumn` and `TrySetColumn` traits for tuples.

use tuple_set::TupleSet;

use crate::{InsertableTable, SetColumn, TrySetColumn};

// The case for 1-element tuples is trivial.
impl<C> SetColumn<C> for (<<C as diesel::Column>::Table as InsertableTable>::InsertableModel,)
where
    C: crate::TypedColumn,
    <C as diesel::Column>::Table: InsertableTable,
    <<C as diesel::Column>::Table as InsertableTable>::InsertableModel: SetColumn<C>,
{
    fn set(&mut self, value: &<C as crate::TypedColumn>::Type) {
        self.0.set(value)
    }
}

// The case for 1-element tuples is trivial.
impl<C> TrySetColumn<C> for (<<C as diesel::Column>::Table as InsertableTable>::InsertableModel,)
where
    C: crate::TypedColumn,
    <C as diesel::Column>::Table: InsertableTable,
    <<C as diesel::Column>::Table as InsertableTable>::InsertableModel: TrySetColumn<C>,
{
    fn try_set(&mut self, value: &<C as crate::TypedColumn>::Type) -> anyhow::Result<()> {
        self.0.try_set(value)
    }
}

macro_rules! impl_set_column {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<C, $head, $($tail),+> SetColumn<C> for ($head, $($tail),+)
		where
			C: crate::TypedColumn,
			<C as diesel::Column>::Table: InsertableTable,
			<<C as diesel::Column>::Table as InsertableTable>::InsertableModel: 'static + SetColumn<C>,
			Self: TupleSet
		{
			fn set(&mut self, value: &<C as crate::TypedColumn>::Type) {
				self.map(|elem: &mut <<C as diesel::Column>::Table as InsertableTable>::InsertableModel| {
					elem.set(value);
				}).unwrap_or_else(|| panic!(
					"Type {insertable_model} was not found in tuple {} for column {}.",
					std::any::type_name::<Self>(),
					std::any::type_name::<C>(),
					insertable_model = std::any::type_name::<<<C as diesel::Column>::Table as InsertableTable>::InsertableModel>(),
				));
			}
		}

		impl<C, $head, $($tail),+> TrySetColumn<C> for ($head, $($tail),+)
		where
			C: crate::TypedColumn,
			<C as diesel::Column>::Table: InsertableTable,
			<<C as diesel::Column>::Table as InsertableTable>::InsertableModel: 'static + TrySetColumn<C>,
			Self: TupleSet
		{
			fn try_set(&mut self, value: &<C as crate::TypedColumn>::Type) -> anyhow::Result<()> {
				self.map(|elem: &mut <<C as diesel::Column>::Table as InsertableTable>::InsertableModel| {
					elem.try_set(value)
				}).unwrap_or_else(|| anyhow::bail!(
					"Type {insertable_model} was not found in tuple {} for column {}.",
					std::any::type_name::<Self>(),
					std::any::type_name::<C>(),
					insertable_model = std::any::type_name::<<<C as diesel::Column>::Table as InsertableTable>::InsertableModel>(),
				))
			}
		}

		impl_set_column!($($tail),+);
	};
}

generate_tuple_impls!(impl_set_column);
