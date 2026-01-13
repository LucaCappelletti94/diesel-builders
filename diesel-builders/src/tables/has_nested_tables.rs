//! Submodule defining and implementing the `HasNestedTables` trait.

use diesel::associations::HasTable;

use super::NestedTables;
use crate::TableExt;

/// Trait for objects that have an associated nested tables structure.
pub trait HasNestedTables {
    /// The nested tables associated with this object.
    type NestedTables: NestedTables;
}

impl HasNestedTables for () {
    type NestedTables = ();
}

impl<T> HasNestedTables for (T,)
where
    T: HasTable<Table: TableExt>,
{
    type NestedTables = (T::Table,);
}

impl<Head, Tail> HasNestedTables for (Head, Tail)
where
    Head: HasTable<Table: TableExt>,
    Tail: HasNestedTables,
    (Head::Table, Tail::NestedTables): NestedTables,
{
    type NestedTables = (Head::Table, Tail::NestedTables);
}
