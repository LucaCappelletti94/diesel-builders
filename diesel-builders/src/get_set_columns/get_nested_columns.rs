//! Submodule providing the `GetNestedColumns` trait.

use crate::{GetColumn, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can get multiple columns.
pub trait GetNestedColumns<CS: NonEmptyNestedProjection> {
    /// Get the values of the specified columns.
    fn get_nested_columns(&self) -> CS::NestedTupleColumnType;
}

impl<C1, T> GetNestedColumns<(C1,)> for T
where
    T: GetColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn get_nested_columns(&self) -> (C1::ColumnType,) {
        (self.get_column(),)
    }
}

impl<CHead, CTail, T> GetNestedColumns<(CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NonEmptyNestedProjection,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: GetColumn<CHead> + GetNestedColumns<CTail>,
{
    #[inline]
    fn get_nested_columns(&self) -> (CHead::ColumnType, CTail::NestedTupleColumnType) {
        (self.get_column(), self.get_nested_columns())
    }
}
