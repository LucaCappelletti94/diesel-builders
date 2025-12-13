//! Submodule providing the `TupleGetNestedColumns` trait.

use crate::{GetColumn, TypedColumn, TypedNestedTuple, columns::NestedColumns};

/// Variant of `GetNestedColumns` for n-uples.
pub trait TupleGetNestedColumns<CS: NestedColumns> {
    /// Get the values of the specified columns as an n-uple.
    fn tuple_get_nested_columns(&self) -> <CS as TypedNestedTuple>::NestedTupleType;
}

impl TupleGetNestedColumns<()> for () {
    #[inline]
    fn tuple_get_nested_columns(&self) {}
}

impl<T1, C1> TupleGetNestedColumns<(C1,)> for (T1,)
where
    T1: GetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn tuple_get_nested_columns(&self) -> <(C1,) as TypedNestedTuple>::NestedTupleType {
        (self.0.get_column(),)
    }
}

impl<Chead, CTail, THead, TTail> TupleGetNestedColumns<(Chead, CTail)> for (THead, TTail)
where
    Chead: TypedColumn,
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    THead: GetColumn<Chead>,
    TTail: TupleGetNestedColumns<CTail>,
{
    #[inline]
    fn tuple_get_nested_columns(&self) -> <(Chead, CTail) as TypedNestedTuple>::NestedTupleType {
        (self.0.get_column(), self.1.tuple_get_nested_columns())
    }
}
