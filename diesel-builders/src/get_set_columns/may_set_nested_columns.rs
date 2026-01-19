//! Trait indicating a builder which may set multiple columns.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{MaySetColumn, TableExt, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder which may set multiple columns.
pub trait MaySetColumns<CS: NonEmptyNestedProjection> {
    /// May set the `nested_values` of the specified columns.
    fn may_set_nested_columns(
        &mut self,
        nested_values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> &mut Self;
}

impl<C1, T> MaySetColumns<(C1,)> for T
where
    T: MaySetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn may_set_nested_columns(&mut self, (value,): (Option<C1::ColumnType>,)) -> &mut Self {
        self.may_set_column(value);
        self
    }
}

impl<CHead, CTail, T> MaySetColumns<(CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: MaySetColumn<CHead> + MaySetColumns<CTail>,
{
    #[inline]
    fn may_set_nested_columns(
        &mut self,
        (head, tail): (
            Option<CHead::ColumnType>,
            <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> &mut Self {
        self.may_set_column(head);
        self.may_set_nested_columns(tail);
        self
    }
}
