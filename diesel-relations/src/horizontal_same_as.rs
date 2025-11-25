//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_additions::{
    ForeignKey, Projection, SingleColumnForeignKey, SingletonForeignKey, Tables, table_addition::HasPrimaryKey
};

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsColumn<
    KeyColumn: SingleColumnForeignKey<<<Self as Column>::Table as Table>::PrimaryKey>,
    HostColumn: Column<Table = KeyColumn::Table>,
>: Column<Table: HasPrimaryKey>
{
}

impl<KeyColumn, HostColumn, ForeignColumn> HorizontalSameAsColumn<KeyColumn, HostColumn>
    for ForeignColumn
where
    KeyColumn: SingleColumnForeignKey<<<Self as Column>::Table as Table>::PrimaryKey>,
    HostColumn: Column<Table = KeyColumn::Table>,
    ForeignColumn: Column<Table: HasPrimaryKey>,
    (KeyColumn, HostColumn):
        ForeignKey<(<<ForeignColumn as Column>::Table as Table>::PrimaryKey, ForeignColumn)>,
{
}

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsKey: SingletonForeignKey {
    /// The set of host columns in the same table which have
    /// an horizontal same-as relationship defined by this key.
    type HostColumns: Projection<Table = Self::Table>;
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type ForeignColumns: Projection<Table = Self::ReferencedTable>;
}

/// A trait for Diesel columns collections that define horizontal same-as
/// relationships.
pub trait HorizontalSameAsKeys: Projection {
    /// The set of referenced tables.
    type ReferencedTables: Tables;
}

macro_rules! impl_horizontal_same_as_keys {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> HorizontalSameAsKeys for ($head,)
		where
			$head: HorizontalSameAsKey,
		{
            type ReferencedTables = (<$head as SingletonForeignKey>::ReferencedTable,);
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> HorizontalSameAsKeys for ($head, $($tail),+)
		where
			$head: HorizontalSameAsKey,
			$($tail: HorizontalSameAsKey<Table = <$head as Column>::Table>),+
		{
            type ReferencedTables = (
                <$head as SingletonForeignKey>::ReferencedTable,
                $(<$tail as SingletonForeignKey>::ReferencedTable),+
            );
		}

		impl_horizontal_same_as_keys!($($tail),+);
	};
}

generate_tuple_impls!(impl_horizontal_same_as_keys);
