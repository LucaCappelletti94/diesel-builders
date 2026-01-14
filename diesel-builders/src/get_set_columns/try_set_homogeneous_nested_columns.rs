//! Trait indicating a builder can set multiple columns.

use crate::{
    OptionalRef, TrySetColumn, TypedColumn, ValidateColumn,
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
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error>;
}

impl<Type, Error, T> TrySetHomogeneousNestedColumns<Type, Error, ()> for T {
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        _value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<Type: Clone, C1, Error, T> TrySetHomogeneousNestedColumns<Type, Error, (C1,)> for T
where
    T: TrySetColumn<C1>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
    C1: TypedColumn<ColumnType: From<Type>>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        if let Some(value) = value.as_optional_ref() {
            self.try_set_column(value.clone())?;
        }
        Ok(self)
    }
}

impl<Error, Type: Clone, CHead, CTail, T>
    TrySetHomogeneousNestedColumns<Type, Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CHead::ColumnType: From<Type>,
    CTail: HomogeneouslyTypedNestedColumns<Type>,
    (CHead, CTail): NonEmptyNestedProjection<
        NestedTupleValueType = (CHead::ValueType, CTail::NestedTupleValueType),
    >,
    T: TrySetColumn<CHead> + TrySetHomogeneousNestedColumns<Type, Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        self.try_set_homogeneous_nested_columns(value)?;
        if let Some(value) = value.as_optional_ref() {
            self.try_set_column(value.clone())?;
        }
        Ok(self)
    }
}
