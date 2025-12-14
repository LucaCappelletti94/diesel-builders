//! Trait indicating a builder can set multiple columns.

use crate::{
    TrySetColumn, TypedColumn, ValidateColumn,
    columns::{HomogeneouslyTypedNestedColumns, NonEmptyNestedProjection},
};

/// Trait indicating a builder can set multiple columns.
pub trait TrySetHomogeneousNestedColumns<Type, Error, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// Set the `nested_values` of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if the value fails validation for any of the columns.
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error>;
}

impl<Type, Error, T> TrySetHomogeneousNestedColumns<Type, Error, ()> for T {
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        _value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<Type: Clone, C1, Error, T> TrySetHomogeneousNestedColumns<Type, Error, (C1,)> for T
where
    T: TrySetColumn<C1>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
    C1: TypedColumn,
    C1::ColumnType: From<Type>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error> {
        let value: Type = value.clone().into();
        Ok(self.try_set_column(value)?)
    }
}

impl<Error, Type: Clone, CHead, CTail, T>
    TrySetHomogeneousNestedColumns<Type, Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CHead::ColumnType: From<Type>,
    CTail: HomogeneouslyTypedNestedColumns<Type>,
    (CHead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (CHead::ColumnType, CTail::NestedTupleType)>,
    T: TrySetColumn<CHead> + TrySetHomogeneousNestedColumns<Type, Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &(impl Into<Type> + Clone),
    ) -> Result<&mut Self, Error> {
        let value: Type = value.clone().into();
        self.try_set_homogeneous_nested_columns(&value)?;
        self.try_set_column(value)?;
        Ok(self)
    }
}
