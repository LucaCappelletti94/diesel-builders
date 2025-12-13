//! Trait indicating a builder which may set multiple columns.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{MaySetColumn, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: NonEmptyNestedProjection> {
    /// May set the `nested_values` of the specified columns.
    fn may_set_nested_columns(
        &mut self,
        nested_values: <CS::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> &mut Self;
}

impl<C1, T> MaySetColumns<(C1,)> for T
where
    T: MaySetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn may_set_nested_columns(&mut self, (value,): (Option<C1::Type>,)) -> &mut Self {
        self.may_set_column(value);
        self
    }
}

impl<Chead, CTail, T> MaySetColumns<(Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (Chead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    T: MaySetColumn<Chead> + MaySetColumns<CTail>,
{
    #[inline]
    fn may_set_nested_columns(
        &mut self,
        (head, tail): (
            Option<Chead::Type>,
            <CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> &mut Self {
        self.may_set_column(head);
        self.may_set_nested_columns(tail);
        self
    }
}
