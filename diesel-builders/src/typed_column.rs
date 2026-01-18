//! Submodule providing the `Typed` trait.

use std::fmt::Debug;

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

impl<V: Debug + Clone + 'static> ValueTyped for DynColumn<V> {
    type ValueType = V;
}

impl<V: Debug + Clone + 'static> ColumnTyped for DynColumn<V> {
    type ColumnType = V;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        let col = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let debug_str = format!("{col:?}");
        assert!(debug_str.contains("DynColumn"));
        assert!(debug_str.contains("table"));
        assert!(debug_str.contains("name"));
    }

    #[test]
    fn test_clone() {
        let col = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let cloned = col;
        assert_eq!(col, cloned);
    }

    #[test]
    fn test_copy() {
        let col = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let copied = col;
        // Since Copy, the original is still valid
        assert_eq!(col, copied);
    }

    #[test]
    fn test_partial_eq() {
        let col1 = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let col2 = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let col3 =
            DynColumn::<i32> { table: "dogs", name: "id", _value_type: std::marker::PhantomData };
        assert_eq!(col1, col2);
        assert_ne!(col1, col3);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let col1 = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let col2 = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let mut set = HashSet::new();
        set.insert(col1);
        assert!(set.contains(&col2));
    }

    #[test]
    fn test_ord() {
        let col1 = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        let col2 = DynColumn::<i32> {
            table: "animals",
            name: "name",
            _value_type: std::marker::PhantomData,
        };
        let col3 =
            DynColumn::<i32> { table: "dogs", name: "id", _value_type: std::marker::PhantomData };
        assert!(col1 < col2); // same table, "id" < "name"
        assert!(col1 < col3); // "animals" < "dogs"
    }

    #[test]
    fn test_display() {
        let col = DynColumn::<i32> {
            table: "animals",
            name: "id",
            _value_type: std::marker::PhantomData,
        };
        assert_eq!(format!("{col}"), "animals.id");
    }
}
