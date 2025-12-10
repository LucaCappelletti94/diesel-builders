//! Trait for builders which may get multiple nested columns.
use tuplities::prelude::{IntoNestedTupleOption, NestedTuplePushFront};

use crate::{MayGetColumn, TypedColumn, TypedNestedTuple, columns::NonEmptyNestedProjection};

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
    fn may_get_nested_columns(
        &self,
    ) -> <<(C1,) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions {
        (self.may_get_column(),)
    }
}

impl<Chead, CTail, T> MayGetNestedColumns<(Chead, CTail)> for T
where
	Chead: TypedColumn,
	CTail: NonEmptyNestedProjection,
	(Chead, CTail): NonEmptyNestedProjection,
	(Chead::Type, CTail::NestedTupleType): IntoNestedTupleOption,
	T: MayGetColumn<Chead> + MayGetNestedColumns<CTail>,
	<CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions: NestedTuplePushFront<
		Option<Chead::Type>,
		Output = <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
	>,
{
	#[inline]
	fn may_get_nested_columns(
		&self,
	) -> <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions
	{
		let head = self.may_get_column();
		let tail = self.may_get_nested_columns();
		tail.nested_push_front(head)
	}
}
