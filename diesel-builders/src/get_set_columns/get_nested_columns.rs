//! Submodule providing the `GetNestedColumns` trait.

use crate::{GetColumn, TypedColumn, columns::NonEmptyNestedProjection};

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
    (Chead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    T: GetColumn<Chead> + GetNestedColumns<CTail>,
{
    #[inline]
    fn get_nested_columns(&self) -> (Chead::Type, CTail::NestedTupleType) {
        (self.get_column(), self.get_nested_columns())
    }
}
