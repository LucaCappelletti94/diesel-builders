//! Variant of `MayGetNestedColumns` for n-uples.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{MayGetColumn, TypedColumn, columns::NestedColumns};

/// Variant of `MayGetNestedColumns` for n-uples.
pub trait TupleMayGetNestedColumns<CS: NestedColumns> {
    /// May get the values of the specified columns as an n-uple.
    fn tuple_may_get_nested_columns(
        &self,
    ) -> <CS::NestedTupleType as IntoNestedTupleOption>::IntoOptions;
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
    fn tuple_may_get_nested_columns(&self) -> (Option<C1::Type>,) {
        (self.0.may_get_column(),)
    }
}

impl<Chead, CTail, THead, TTail> TupleMayGetNestedColumns<(Chead, CTail)> for (THead, TTail)
where
    Chead: TypedColumn,
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    THead: MayGetColumn<Chead>,
    TTail: TupleMayGetNestedColumns<CTail>,
{
    #[inline]
    fn tuple_may_get_nested_columns(
        &self,
    ) -> (
        Option<Chead::Type>,
        <CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) {
        (
            self.0.may_get_column(),
            self.1.tuple_may_get_nested_columns(),
        )
    }
}
