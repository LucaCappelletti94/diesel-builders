//! Trait indicating a builder can set multiple columns.

use crate::{
    OptionalRef, SetColumn, TypedColumn,
    columns::{HomogeneouslyTypedNestedColumns, NonEmptyNestedProjection},
};

/// Trait indicating a builder can set multiple columns.
pub trait SetHomogeneousNestedColumns<Type, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// Set the `nested_values` of the specified columns.
    fn set_homogeneous_nested_columns(&mut self, value: &impl OptionalRef<Type>) -> &mut Self;
}

impl<Type, T> SetHomogeneousNestedColumns<Type, ()> for T {
    #[inline]
    fn set_homogeneous_nested_columns(&mut self, _value: &impl OptionalRef<Type>) -> &mut Self {
        self
    }
}

impl<Type: Clone, C1, T> SetHomogeneousNestedColumns<Type, (C1,)> for T
where
    T: SetColumn<C1>,
    C1: TypedColumn,
    C1::ColumnType: From<Type>,
{
    #[inline]
    fn set_homogeneous_nested_columns(&mut self, value: &impl OptionalRef<Type>) -> &mut Self {
        if let Some(value) = value.as_optional_ref() {
            self.set_column(value.clone());
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
    fn set_homogeneous_nested_columns(&mut self, value: &impl OptionalRef<Type>) -> &mut Self {
        self.set_homogeneous_nested_columns(value);
        if let Some(value) = value.as_optional_ref() {
            self.set_column(value.clone());
        }
        self
    }
}
