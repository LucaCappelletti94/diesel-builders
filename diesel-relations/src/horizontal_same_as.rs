//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_additions::ForeignKey;

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAs: Column
where
    <<Self::AssociatedColumn as Column>::Table as Table>::PrimaryKey:
        Column<Table = <Self::AssociatedColumn as Column>::Table>,
    (Self::KeyColumn,):
        ForeignKey<(<<Self::AssociatedColumn as Column>::Table as Table>::PrimaryKey,)>,
    (Self::KeyColumn, Self): ForeignKey<(
        <<Self::AssociatedColumn as Column>::Table as Table>::PrimaryKey,
        Self::AssociatedColumn,
    )>,
{
    /// A column can have only one associated column for horizontal same-as.
    type AssociatedColumn: Column;
    /// The key column that defines the horizontal same-as relationship.
    type KeyColumn: Column<Table = Self::Table>;
}
