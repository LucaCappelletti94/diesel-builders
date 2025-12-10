//! Submodule providing the `TupleGetNestedColumns` trait.

use tuplities::prelude::{FlattenNestedTuple, NestedTuplePushFront};

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

impl<Chead, CTail, T> TupleGetNestedColumns<(Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns,
    (Chead::Type, CTail::NestedTupleType): FlattenNestedTuple,
    T: GetColumn<Chead> + TupleGetNestedColumns<CTail>,
    CTail::NestedTupleType: NestedTuplePushFront<
            Chead::Type,
            Output = <(Chead, CTail) as TypedNestedTuple>::NestedTupleType,
        >,
{
    #[inline]
    fn tuple_get_nested_columns(&self) -> <(Chead, CTail) as TypedNestedTuple>::NestedTupleType {
        let head = self.get_column();
        let tail = self.tuple_get_nested_columns();
        tail.nested_push_front(head)
    }
}
