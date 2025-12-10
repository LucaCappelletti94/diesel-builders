//! Trait indicating a builder can set multiple columns.

use tuplities::prelude::NestedTuplePopFront;

use crate::{SetColumn, TypedColumn, TypedNestedTuple, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can set multiple columns.
pub trait SetNestedColumns<CS: NonEmptyNestedProjection> {
    /// Set the `nested_values` of the specified columns.
    fn set_nested_columns(&mut self, nested_values: CS::NestedTupleType) -> &mut Self;
}

impl<C1, T> SetNestedColumns<(C1,)> for T
where
    T: SetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn set_nested_columns(&mut self, nested_values: (C1::Type,)) -> &mut Self {
        self.set_column(nested_values.0)
    }
}

impl<Chead, CTail, T> SetNestedColumns<(Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (Chead, CTail): NonEmptyNestedProjection,
    T: SetColumn<Chead> + SetNestedColumns<CTail>,
    <(Chead, CTail) as TypedNestedTuple>::NestedTupleType:
        NestedTuplePopFront<Front = Chead::Type, Tail = CTail::NestedTupleType>,
{
    #[inline]
    fn set_nested_columns(
        &mut self,
        nested_values: <(Chead, CTail) as TypedNestedTuple>::NestedTupleType,
    ) -> &mut Self {
        let (head, tail) = nested_values.nested_pop_front();
        self.set_column(head);
        self.set_nested_columns(tail);
        self
    }
}
