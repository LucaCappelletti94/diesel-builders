//! Submodule providing the `GetNestedColumns` trait.

use tuplities::prelude::NestedTupleRef;

use crate::{GetColumn, TypedColumn, columns::NonEmptyNestedProjection};

/// Trait indicating a builder can get multiple columns.
pub trait GetNestedColumns<CS: NonEmptyNestedProjection> {
    /// Get the values of the specified columns.
    fn get_nested_columns(&self) -> CS::NestedTupleColumnType;
    /// Get the references of the specified columns.
    fn get_nested_column_refs(&self) -> <CS::NestedTupleColumnType as NestedTupleRef>::Ref<'_>;
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
    #[inline]
    fn get_nested_column_refs(&self) -> (&C1::ColumnType,) {
        (self.get_column_ref(),)
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

    #[inline]
    fn get_nested_column_refs(
        &self,
    ) -> (&CHead::ColumnType, <CTail::NestedTupleColumnType as NestedTupleRef>::Ref<'_>) {
        (self.get_column_ref(), self.get_nested_column_refs())
    }
}
