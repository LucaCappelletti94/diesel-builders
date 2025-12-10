//! Trait indicating a builder which may set multiple columns.

use tuplities::prelude::{IntoNestedTupleOption, TuplePopFront};

use crate::{MaySetColumn, TypedColumn, TypedNestedTuple, columns::NestedColumns};

/// Trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: NestedColumns> {
    /// May set the `nested_values` of the specified columns.
    fn may_set_nested_columns(
        &mut self,
        nested_values: <CS::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> &mut Self;
}

impl<T> MaySetColumns<()> for T {
    #[inline]
    fn may_set_nested_columns(&mut self, _nested_values: ()) -> &mut Self {
        self
    }
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
    CTail: NestedColumns,
    (Chead, CTail): NestedColumns,
    T: MaySetColumn<Chead> + MaySetColumns<CTail>,
    <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions:
        TuplePopFront<
                Front = Option<Chead::Type>,
                Tail = (<CTail::NestedTupleType as IntoNestedTupleOption>::IntoOptions,),
            >,
{
    #[inline]
    fn may_set_nested_columns(
        &mut self,
        nested_values: <<(Chead, CTail) as TypedNestedTuple>::NestedTupleType as IntoNestedTupleOption>::IntoOptions,
    ) -> &mut Self {
        let (head, (tail,)) = nested_values.pop_front();
        self.may_set_column(head);
        self.may_set_nested_columns(tail);
        self
    }
}
