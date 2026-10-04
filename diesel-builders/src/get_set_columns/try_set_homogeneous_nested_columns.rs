//! Trait indicating a builder can set multiple columns.

use crate::{
    OptionalRef, TableExt, TypedColumn, ValidateColumn,
    columns::{HomogeneouslyTypedNestedColumns, NonEmptyNestedProjection},
    mutation::{MutationContext, PrepareColumn, PrepareHomogeneous},
};

/// Trait indicating a builder can set multiple columns.
pub trait TrySetHomogeneousNestedColumns<Type, Error, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// Set the `nested_values` of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if the value fails validation for any of the columns.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{MayGetColumn, TrySetHomogeneousNestedColumns};
    /// use schema::{ValidationError, users};
    ///
    /// let mut values = user_values("Ada", 18, None);
    /// TrySetHomogeneousNestedColumns::<i32, ValidationError, (users::age,)>::try_set_homogeneous_nested_columns(
    ///     &mut values,
    ///     &20,
    /// )?;
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(20));
    /// assert_eq!(
    ///     TrySetHomogeneousNestedColumns::<i32, ValidationError, (users::age,)>::try_set_homogeneous_nested_columns(
    ///         &mut values,
    ///         &17,
    ///     )
    ///     .err(),
    ///     Some(ValidationError::AgeTooYoung)
    /// );
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(20));
    ///
    /// // A missing value, such as an unattached optional same-as target,
    /// // leaves every column in the group untouched.
    /// TrySetHomogeneousNestedColumns::<i32, ValidationError, (users::age,)>::try_set_homogeneous_nested_columns(
    ///     &mut values,
    ///     &None::<i32>,
    /// )?;
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(20));
    /// # Ok(())
    /// # }
    /// ```
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
    T: PrepareColumn<C1>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
    C1: TypedColumn<ColumnType: From<Type>, Table: TableExt>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared = <T as PrepareHomogeneous<Error, Type, (C1,)>>::prepare_homogeneous(
            self, value, &context,
        )?;
        <T as PrepareHomogeneous<Error, Type, (C1,)>>::apply_homogeneous(self, prepared);
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
            NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType),
        >,
    T: PrepareColumn<CHead> + PrepareHomogeneous<Error, Type, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn try_set_homogeneous_nested_columns(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared = <T as PrepareHomogeneous<Error, Type, (CHead, CTail)>>::prepare_homogeneous(
            self, value, &context,
        )?;
        <T as PrepareHomogeneous<Error, Type, (CHead, CTail)>>::apply_homogeneous(self, prepared);
        Ok(self)
    }
}
