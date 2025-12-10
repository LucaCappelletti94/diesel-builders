//! Variant of `MayGetNestedColumns` for n-uples.

use tuplities::prelude::{IntoNestedTupleOption, NestedTuplePushFront};

use crate::{MayGetColumn, TypedColumn, TypedNestedTuple, columns::NestedColumns};

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
    fn tuple_may_get_nested_columns(
        &self,
    ) -> <<(C1,) as crate::TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions
    {
        (self.0.may_get_column(),)
    }
}

impl<Chead, CTail, T> TupleMayGetNestedColumns<(Chead, CTail)> for T
where
    Chead: TypedColumn,
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns,
    (Chead::Type, CTail::NestedTupleType): IntoNestedTupleOption,
    T: MayGetColumn<Chead> + TupleMayGetNestedColumns<CTail>,
    <CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions: NestedTuplePushFront<
            Option<Chead::Type>,
            Output = <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions
        >,
{
    #[inline]
	fn tuple_may_get_nested_columns(
		&self,
	) -> <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions
	{
        let head = self.may_get_column();
        let tail = self.tuple_may_get_nested_columns();
        tail.nested_push_front(head)
    }
}
