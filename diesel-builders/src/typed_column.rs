//! Submodule providing the `Typed` trait.

use crate::Typed;

/// Trait representing an object with an associated type.
pub trait TypedColumn: diesel::Column<Table: Default> + Typed + Default + Copy {}

impl<T> TypedColumn for T where T: diesel::Column<Table: Default> + Typed + Default + Copy {}

/// Trait variant of `TypedColumn` which is dyn safe.
pub trait DynTypedColumn {
    /// The Value type associated with this column.
    type ValueType;

    /// Returns the name of the column.
    fn column_name(&self) -> &'static str;
}

impl<C> DynTypedColumn for C
where
    C: TypedColumn,
{
    type ValueType = C::ValueType;

    fn column_name(&self) -> &'static str {
        C::NAME
    }
}
