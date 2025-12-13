//! Trait indicating a builder can set multiple columns.

use crate::{NestedColumns, SetColumn, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can set multiple columns.
pub trait SetNestedColumns<CS: NestedColumns> {
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
    (Chead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    T: SetColumn<Chead> + SetNestedColumns<CTail>,
{
    #[inline]
    fn set_nested_columns(
        &mut self,
        (head, tail): (Chead::Type, CTail::NestedTupleType),
    ) -> &mut Self {
        self.set_column(head);
        self.set_nested_columns(tail);
        self
    }
}
