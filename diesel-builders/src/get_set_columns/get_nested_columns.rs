//! Submodule providing the `GetNestedColumns` trait.

use tuplities::prelude::NestedTuplePushFront;

use crate::{GetColumn, TypedColumn, TypedNestedTuple, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can get multiple columns.
pub trait GetNestedColumns<CS: NonEmptyNestedProjection> {
    /// Get the values of the specified columns.
    fn get_nested_columns(&self) -> CS::NestedTupleType;
}

impl<C1, T> GetNestedColumns<(C1,)> for T
where
    T: GetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn get_nested_columns(&self) -> (C1::Type,) {
        (self.get_column(),)
    }
}

impl<Chead, CTail, T> GetNestedColumns<(Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (Chead, CTail): NonEmptyNestedProjection,
    T: GetColumn<Chead> + GetNestedColumns<CTail>,
    CTail::NestedTupleType: NestedTuplePushFront<
            Chead::Type,
            Output = <(Chead, CTail) as TypedNestedTuple>::NestedTupleType,
        >,
{
    #[inline]
    fn get_nested_columns(&self) -> <(Chead, CTail) as TypedNestedTuple>::NestedTupleType {
        let head = self.get_column();
        let tail = self.get_nested_columns();
        tail.nested_push_front(head)
    }
}
