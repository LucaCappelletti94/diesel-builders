//! Submodule defining and implementing the `Tables` trait.

use diesel::associations::HasTable;

use crate::{DefaultTuple, InsertableTableModel, TableAddition, TableModel};

/// A trait representing a collection of Diesel tables.
pub trait Tables {
    /// The n-uple of models corresponding to the tables in this collection.
    type Models: TableModels;
    /// The n-uple of insertable models corresponding to the tables in this
    /// collection.
    type InsertableModels: InsertableTableModels;
}

impl Tables for () {
    type Models = ();
    type InsertableModels = ();
}

/// Trait representing an n-uple of TableModels.
pub trait TableModels {
    /// The n-uple of tables corresponding to these models.
    type Tables: Tables<Models = Self>;
}

impl TableModels for () {
    type Tables = ();
}

/// Trait representing an n-uple of InsertableTableModels.
pub trait InsertableTableModels: Sized + DefaultTuple {
    /// The n-uple of tables corresponding to these insertable models.
    type Tables: Tables<InsertableModels = Self>;
}

impl InsertableTableModels for () {
    type Tables = ();
}

macro_rules! impl_tables {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> Tables for ($head,)
		where
			$head: TableAddition
		{
			type Models = (<$head as TableAddition>::Model,);
			type InsertableModels = (<$head as TableAddition>::InsertableModel,);
		}

		impl<$head> TableModels for ($head,)
		where
			$head: TableModel
		{
			type Tables = (<$head as HasTable>::Table,);
		}

		impl<$head> InsertableTableModels for ($head,)
		where
			$head: InsertableTableModel
		{
			type Tables = (<$head as HasTable>::Table,);
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> Tables for ($head, $($tail),+)
		where
			$head: TableAddition,
			$($tail: TableAddition),+
		{
			type Models = (<$head as TableAddition>::Model, $(<$tail as TableAddition>::Model),+);
			type InsertableModels = (<$head as TableAddition>::InsertableModel, $(<$tail as TableAddition>::InsertableModel),+);
		}

		impl<$head, $($tail),+> TableModels for ($head, $($tail),+)
		where
			$head: TableModel,
			$($tail: TableModel),+
		{
			type Tables = (<$head as HasTable>::Table, $(<$tail as HasTable>::Table),+);
		}
		impl<$head, $($tail),+> InsertableTableModels for ($head, $($tail),+)
		where
			$head: InsertableTableModel,
			$($tail: InsertableTableModel),+
		{
			type Tables = (<$head as HasTable>::Table, $(<$tail as HasTable>::Table),+);
		}

		impl_tables!($($tail),+);
	};
}

generate_tuple_impls!(impl_tables);
