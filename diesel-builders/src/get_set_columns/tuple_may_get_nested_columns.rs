//! Variant of `MayGetNestedColumns` for n-uples.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{MayGetColumn, TypedColumn, columns::NestedColumns};

/// Variant of `MayGetNestedColumns` for n-uples.
pub trait TupleMayGetNestedColumns<CS: NestedColumns> {
    /// May get the values of the specified columns as an n-uple.
    fn tuple_may_get_nested_columns(
        &self,
    ) -> <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions;
}

impl TupleMayGetNestedColumns<()> for () {
    #[inline]
    fn tuple_may_get_nested_columns(&self) {}
}

impl<T1, C1> TupleMayGetNestedColumns<(C1,)> for (T1,)
where
    T1: MayGetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn tuple_may_get_nested_columns(&self) -> (Option<C1::ColumnType>,) {
        (self.0.may_get_column(),)
    }
}

impl<CHead, CTail, THead, TTail> TupleMayGetNestedColumns<(CHead, CTail)> for (THead, TTail)
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    THead: MayGetColumn<CHead>,
    TTail: TupleMayGetNestedColumns<CTail>,
{
    #[inline]
    fn tuple_may_get_nested_columns(
        &self,
    ) -> (
        Option<CHead::ColumnType>,
        <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) {
        (self.0.may_get_column(), self.1.tuple_may_get_nested_columns())
    }
}
