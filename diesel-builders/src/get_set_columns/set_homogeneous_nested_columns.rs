//! Trait indicating a builder can set multiple columns.

use crate::{
    SetColumn, TypedColumn,
    columns::{HomogeneouslyTypedNestedColumns, NonEmptyNestedProjection},
};

/// Trait indicating a builder can set multiple columns.
pub trait SetHomogeneousNestedColumns<Type, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// Set the `nested_values` of the specified columns.
    fn set_homogeneous_nested_columns(
        &mut self,
        value: &(impl Into<Option<Type>> + Clone),
    ) -> &mut Self;
}

impl<Type, T> SetHomogeneousNestedColumns<Type, ()> for T {
    #[inline]
    fn set_homogeneous_nested_columns(
        &mut self,
        _value: &(impl Into<Option<Type>> + Clone),
    ) -> &mut Self {
        self
    }
}

impl<Type, C1, T> SetHomogeneousNestedColumns<Type, (C1,)> for T
where
    T: SetColumn<C1>,
    C1: TypedColumn,
    C1::ColumnType: From<Type>,
{
    #[inline]
    fn set_homogeneous_nested_columns(
        &mut self,
        value: &(impl Into<Option<Type>> + Clone),
    ) -> &mut Self {
        if let Some(value) = value.clone().into() {
            self.set_column(value);
        }
        self
    }
}

impl<Type: Clone, CHead, CTail, T> SetHomogeneousNestedColumns<Type, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: HomogeneouslyTypedNestedColumns<Type>,
    CHead::ColumnType: From<Type>,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
    >,
    T: SetColumn<CHead> + SetHomogeneousNestedColumns<Type, CTail>,
{
    #[inline]
    fn set_homogeneous_nested_columns(
        &mut self,
        value: &(impl Into<Option<Type>> + Clone),
    ) -> &mut Self {
        let value: Option<Type> = value.clone().into();
        self.set_homogeneous_nested_columns(&value);
        if let Some(value) = value.clone() {
            self.set_column(value);
        }
        self
    }
}
