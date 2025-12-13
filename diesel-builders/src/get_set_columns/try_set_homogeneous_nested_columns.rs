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
    fn try_set_homogeneous_nested_columns(&mut self, value: Type) -> Result<&mut Self, Error>;
}

impl<Type, Error, T> TrySetHomogeneousNestedColumns<Type, Error, ()> for T {
    #[inline]
    fn try_set_homogeneous_nested_columns(&mut self, _value: Type) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<C1, Error, T> TrySetHomogeneousNestedColumns<C1::Type, Error, (C1,)> for T
where
    T: TrySetColumn<C1>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
    C1: TypedColumn,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(&mut self, value: C1::Type) -> Result<&mut Self, Error> {
        Ok(self.try_set_column(value)?)
    }
}

impl<Error, Chead, CTail, T> TrySetHomogeneousNestedColumns<Chead::Type, Error, (Chead, CTail)>
    for T
where
    Chead: TypedColumn,
    CTail: HomogeneouslyTypedNestedColumns<Chead::Type>,
    (Chead, CTail):
        NonEmptyNestedProjection<NestedTupleType = (Chead::Type, CTail::NestedTupleType)>,
    T: TrySetColumn<Chead> + TrySetHomogeneousNestedColumns<Chead::Type, Error, CTail>,
    Error: From<<T as ValidateColumn<Chead>>::Error>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: Chead::Type,
    ) -> Result<&mut Self, Error> {
        self.try_set_column(value.clone())?;
        self.try_set_homogeneous_nested_columns(value)?;
        Ok(self)
    }
}
