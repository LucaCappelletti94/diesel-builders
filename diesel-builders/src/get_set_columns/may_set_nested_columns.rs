//! Trait indicating a builder which may set multiple columns.

use tuplities::prelude::{IntoNestedTupleOption, NestedTuplePopFront};

use crate::{MaySetColumn, TypedColumn, TypedNestedTuple, columns::NonEmptyNestedProjection};

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
    (Chead, CTail): NonEmptyNestedProjection,
    T: MaySetColumn<Chead> + MaySetColumns<CTail>,
    <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions:
        NestedTuplePopFront<
                Front = Option<Chead::Type>,
                Tail = <CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
            >,
{
    #[inline]
    fn may_set_nested_columns(
        &mut self,
        nested_values: <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> &mut Self {
        let (head, tail) = nested_values.nested_pop_front();
        self.may_set_column(head);
        self.may_set_nested_columns(tail);
        self
    }
}
