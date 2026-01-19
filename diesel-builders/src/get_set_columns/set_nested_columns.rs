//! Trait indicating a builder can set multiple columns.

use crate::{NestedColumns, SetColumn, TableExt, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can set multiple columns.
pub trait SetNestedColumns<CS: NestedColumns> {
    /// Set the `nested_values` of the specified columns.
    fn set_nested_columns(&mut self, nested_values: CS::NestedTupleColumnType) -> &mut Self;
}

impl<C1, T> SetNestedColumns<(C1,)> for T
where
    T: SetColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn set_nested_columns(&mut self, nested_values: (C1::ColumnType,)) -> &mut Self {
        self.set_column(nested_values.0)
    }
}

impl<CHead, CTail, T> SetNestedColumns<(CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: SetColumn<CHead> + SetNestedColumns<CTail>,
{
    #[inline]
    fn set_nested_columns(
        &mut self,
        (head, tail): (CHead::ColumnType, CTail::NestedTupleColumnType),
    ) -> &mut Self {
        self.set_column(head);
        self.set_nested_columns(tail);
        self
    }
}
