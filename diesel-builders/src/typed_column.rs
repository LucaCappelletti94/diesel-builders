//! Submodule providing the `Typed` trait.

use diesel::Column;

use crate::{ColumnTyped, TableExt, ValueTyped};

/// Trait representing an object with an associated type.
pub trait TypedColumn: diesel::Column<Table: Default> + ColumnTyped + Default + Copy {}
impl<T> TypedColumn for T where T: diesel::Column<Table: Default> + ColumnTyped + Default + Copy {}

/// A dynamic column type placeholder.
pub struct DynColumn<V> {
    /// The table of the column.
    table: &'static str,
    /// The name of the column.
    name: &'static str,
    /// Phantom data for the value type.
    _value_type: std::marker::PhantomData<V>,
}

impl<V> std::fmt::Debug for DynColumn<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynColumn").field("table", &self.table).field("name", &self.name).finish()
    }
}

impl<V> core::clone::Clone for DynColumn<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V> core::marker::Copy for DynColumn<V> {}

impl<V> std::cmp::PartialEq for DynColumn<V> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table && self.name == other.name
    }
}

impl<V> std::cmp::Eq for DynColumn<V> {}

impl<V> core::hash::Hash for DynColumn<V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.table.hash(state);
        self.name.hash(state);
    }
}

impl<V> core::cmp::PartialOrd for DynColumn<V> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<V> core::cmp::Ord for DynColumn<V> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.table.cmp(other.table) {
            std::cmp::Ordering::Equal => self.name.cmp(other.name),
            ord => ord,
        }
    }
}

impl<V> std::fmt::Display for DynColumn<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.table, self.name)
    }
}

impl<V: Clone + 'static> ValueTyped for DynColumn<V> {
    type ValueType = V;
}

impl<C> From<C> for DynColumn<C::ValueType>
where
    C: TypedColumn + ValueTyped + Column<Table: TableExt>,
{
    fn from(_: C) -> Self {
        DynColumn {
            table: <C::Table as TableExt>::TABLE_NAME,
            name: C::NAME,
            _value_type: std::marker::PhantomData,
        }
    }
}

impl<V> DynColumn<V> {
    /// Returns the name of the column.
    #[must_use]
    pub fn column_name(&self) -> &'static str {
        self.name
    }

    /// Returns a reference to the table of the column.
    #[must_use]
    pub fn table_name(&self) -> &'static str {
        self.table
    }
}
