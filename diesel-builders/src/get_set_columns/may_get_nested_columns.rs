//! Trait for builders which may get multiple nested columns.
use tuplities::prelude::IntoNestedTupleOption;

use crate::{MayGetColumn, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder which may get multiple columns.
pub trait MayGetNestedColumns<CS: NonEmptyNestedProjection> {
    /// May get the owned values of the specified columns.
    fn may_get_nested_columns(&self)
    -> <CS::NestedTupleType as IntoNestedTupleOption>::IntoOptions;
}

impl<C1, T> MayGetNestedColumns<(C1,)> for T
where
    T: MayGetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn may_get_nested_columns(&self) -> (Option<C1::Type>,) {
        (self.may_get_column(),)
    }
}

impl<Chead, CTail, T> MayGetNestedColumns<(Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (Chead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    T: MayGetColumn<Chead> + MayGetNestedColumns<CTail>,
{
    #[inline]
    fn may_get_nested_columns(
        &self,
    ) -> (
        Option<Chead::Type>,
        <CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) {
        (self.may_get_column(), self.may_get_nested_columns())
    }
}
