//! Submodule providing the `Typed` trait.

use diesel::{Column, Table};

use crate::{
    ColumnTyped, ValueTyped,
    columns::{HomogeneouslyTypedNestedColumns, NonEmptyNestedProjection},
};

/// Trait representing an object with an associated type.
pub trait TypedColumn: diesel::Column<Table: Default> + ColumnTyped + Default + Copy {}
impl<T> TypedColumn for T where T: diesel::Column<Table: Default> + ColumnTyped + Default + Copy {}

/// Trait variant of `TypedColumn` which is dyn safe.
pub trait DynTypedColumn: ValueTyped {
    /// The table from which this column originates.
    type Table: diesel::Table + NestedColumnsByValueType<Self::ValueType>;

    /// Returns the name of the column.
    fn column_name(&self) -> &'static str;
}

impl<C> DynTypedColumn for C
where
    C: Column<Table: NestedColumnsByValueType<C::ValueType>> + ValueTyped + ?Sized,
{
    type Table = C::Table;

    fn column_name(&self) -> &'static str {
        C::NAME
    }
}

/// Trait for Diesel Tables grouping typed columns by their ValueType.
pub trait NestedColumnsByValueType<ValueType>: Table {
    /// Associated nested columns type.
    type NestedColumns: HomogeneouslyTypedNestedColumns<ValueType>
        + NonEmptyNestedProjection<Table = Self>;
}
