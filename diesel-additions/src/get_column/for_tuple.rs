//! Submodule implementing the `GetColumn` and `MayGetColumn` traits for tuples.

use tuple_set::TupleSet;

use crate::{GetColumn, InsertableTable, MayGetColumn};

// The case for 1-element tuples is trivial.
impl<C> GetColumn<C> for (<<C as diesel::Column>::Table as InsertableTable>::InsertableModel,)
where
    C: crate::TypedColumn,
    <C as diesel::Column>::Table: InsertableTable,
    <<C as diesel::Column>::Table as InsertableTable>::InsertableModel: GetColumn<C>,
{
    fn get(&self) -> &<C as crate::TypedColumn>::Type {
        self.0.get()
    }
}

// The case for 1-element tuples is trivial.
impl<C> MayGetColumn<C> for (<<C as diesel::Column>::Table as InsertableTable>::InsertableModel,)
where
    C: crate::TypedColumn,
    <C as diesel::Column>::Table: InsertableTable,
    <<C as diesel::Column>::Table as InsertableTable>::InsertableModel: MayGetColumn<C>,
{
    fn maybe_get(&self) -> Option<&<C as crate::TypedColumn>::Type> {
        self.0.maybe_get()
    }
}

macro_rules! impl_get_column {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<C, $head, $($tail),+> GetColumn<C> for ($head, $($tail),+)
		where
			C: crate::TypedColumn,
			<C as diesel::Column>::Table: InsertableTable,
			<<C as diesel::Column>::Table as InsertableTable>::InsertableModel: 'static + GetColumn<C>,
			Self: TupleSet
		{
			fn get(&self) -> &<C as crate::TypedColumn>::Type {
				<Self as TupleSet>::get::<<<C as diesel::Column>::Table as InsertableTable>::InsertableModel>(self).unwrap_or_else(|| panic!(
					"Type {insertable_model} was not found in tuple {} for column {}.",
					std::any::type_name::<Self>(),
					std::any::type_name::<C>(),
					insertable_model = std::any::type_name::<<<C as diesel::Column>::Table as InsertableTable>::InsertableModel>(),
				)).get()
			}
		}

		impl<C, $head, $($tail),+> MayGetColumn<C> for ($head, $($tail),+)
		where
			C: crate::TypedColumn,
			<C as diesel::Column>::Table: InsertableTable,
			<<C as diesel::Column>::Table as InsertableTable>::InsertableModel: 'static + MayGetColumn<C>,
			Self: TupleSet
		{
			fn maybe_get(&self) -> Option<&<C as crate::TypedColumn>::Type> {
				<Self as TupleSet>::get::<<<C as diesel::Column>::Table as InsertableTable>::InsertableModel>(self).unwrap_or_else(|| panic!(
					"Type {insertable_model} was not found in tuple {} for column {}.",
					std::any::type_name::<Self>(),
					std::any::type_name::<C>(),
					insertable_model = std::any::type_name::<<<C as diesel::Column>::Table as InsertableTable>::InsertableModel>(),
				)).maybe_get()
			}
		}

		impl_get_column!($($tail),+);
	};
}

generate_tuple_impls!(impl_get_column);
