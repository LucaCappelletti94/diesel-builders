//! Trait for fallibly setting multiple nested columns.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{
    OptionalRef, TableExt, TypedColumn, TypedNestedTuple, ValidateColumn,
    columns::NestedColumns,
    mutation::{MutationContext, PrepareColumn, PrepareColumns},
};

/// Trait indicating a builder can validate multiple nested columns.
pub trait ValidateNestedColumns<Error, CS: NestedColumns> {
    /// Validate the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column fails validation.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::ValidateNestedColumns;
    /// use schema::{ValidationError, users};
    ///
    /// let values = user_values("Ada", 20, Some("Ace"));
    /// ValidateNestedColumns::<ValidationError, (users::age, (users::nickname,))>::validate_nested_columns(
    ///     &values,
    ///     &(20, (Some("Ace".to_string()),)),
    /// )?;
    /// assert_eq!(
    ///     ValidateNestedColumns::<ValidationError, (users::age, (users::name,))>::validate_nested_columns(
    ///         &values,
    ///         &(17, ("Ada".to_string(),)),
    ///     )
    ///     .err(),
    ///     Some(ValidationError::AgeTooYoung)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn validate_nested_columns(&self, values: &CS::NestedTupleColumnType) -> Result<(), Error>;
}

impl<C1, T, Error> ValidateNestedColumns<Error, (C1,)> for T
where
    Error: From<<T as ValidateColumn<C1>>::Error>,
    T: ValidateColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn validate_nested_columns(&self, (head,): &(C1::ColumnType,)) -> Result<(), Error> {
        if let Some(head) = head.as_optional_ref() {
            <Self as ValidateColumn<C1>>::validate_column(head)?;
        }
        Ok(())
    }
}

impl<CHead, CTail, T, Error> ValidateNestedColumns<Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: ValidateColumn<CHead> + ValidateNestedColumns<Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn validate_nested_columns(
        &self,
        (head, tail): &(CHead::ColumnType, CTail::NestedTupleColumnType),
    ) -> Result<(), Error> {
        if let Some(head) = head.as_optional_ref() {
            <Self as ValidateColumn<CHead>>::validate_column(head)?;
        }
        self.validate_nested_columns(tail)?;
        Ok(())
    }
}

/// Trait indicating a builder can validate multiple optional nested columns.
pub trait MayValidateNestedColumns<Error, CS: NestedColumns> {
    /// Validate the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column fails validation.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::MayValidateNestedColumns;
    /// use schema::{ValidationError, users};
    ///
    /// let values = user_values("Ada", 20, None);
    /// let missing: (Option<Option<String>>,) = (None,);
    /// MayValidateNestedColumns::<ValidationError, (users::nickname,)>::may_validate_nested_columns(
    ///     &values,
    ///     &missing,
    /// )?;
    /// assert_eq!(
    ///     MayValidateNestedColumns::<ValidationError, (users::nickname,)>::may_validate_nested_columns(
    ///         &values,
    ///         &(Some(Some(String::new())),),
    ///     )
    ///     .err(),
    ///     Some(ValidationError::EmptyNickname)
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn may_validate_nested_columns(
        &self,
        values: &<CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<(), Error>;
}

impl<C1, T, Error> MayValidateNestedColumns<Error, (C1,)> for T
where
    Error: From<<T as ValidateColumn<C1>>::Error>,
    T: ValidateColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
{
    #[inline]
    fn may_validate_nested_columns(
        &self,
        (head,): &(Option<C1::ColumnType>,),
    ) -> Result<(), Error> {
        if let Some(v1) = head.as_ref().and_then(|v| v.as_optional_ref()) {
            <Self as ValidateColumn<C1>>::validate_column(v1)?;
        }
        Ok(())
    }
}

impl<CHead, CTail, T, Error> MayValidateNestedColumns<Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: ValidateColumn<CHead> + MayValidateNestedColumns<Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn may_validate_nested_columns(
        &self,
        (head, tail): &(
            Option<CHead::ColumnType>,
            <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> Result<(), Error> {
        if let Some(head) = head.as_ref().and_then(|v| v.as_optional_ref()) {
            <Self as ValidateColumn<CHead>>::validate_column(head)?;
        }
        self.may_validate_nested_columns(tail)?;
        Ok(())
    }
}

/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetNestedColumns<Error, CS: NestedColumns> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{MayGetColumn, TrySetNestedColumns};
    /// use schema::{ValidationError, users};
    ///
    /// let mut values = user_values("Ada", 18, None);
    /// TrySetNestedColumns::<ValidationError, (users::name, (users::age,))>::try_set_nested_columns(
    ///     &mut values,
    ///     ("Grace".to_string(), (30,)),
    /// )?;
    /// assert_eq!(
    ///     MayGetColumn::<users::name>::may_get_column(&values),
    ///     Some("Grace".to_string())
    /// );
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(30));
    /// assert_eq!(
    ///     TrySetNestedColumns::<ValidationError, (users::name, (users::age,))>::try_set_nested_columns(
    ///         &mut values,
    ///         ("Ada".to_string(), (17,)),
    ///     )
    ///     .err(),
    ///     Some(ValidationError::AgeTooYoung)
    /// );
    /// assert_eq!(
    ///     MayGetColumn::<users::name>::may_get_column(&values),
    ///     Some("Grace".to_string())
    /// );
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(30));
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_nested_columns(
        &mut self,
        values: CS::NestedTupleColumnType,
    ) -> Result<&mut Self, Error>;
}

impl<T, Error> TrySetNestedColumns<Error, ()> for T {
    #[inline]
    fn try_set_nested_columns(&mut self, _values: ()) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<C1, T, Error> TrySetNestedColumns<Error, (C1,)> for T
where
    T: PrepareColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
{
    #[inline]
    fn try_set_nested_columns(&mut self, values: (C1::ColumnType,)) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared = <T as PrepareColumns<Error, (C1,)>>::prepare_columns(self, values, &context)
            .map_err(|(_, error)| error)?;
        <T as PrepareColumns<Error, (C1,)>>::apply_columns(self, prepared);
        Ok(self)
    }
}

impl<CHead, CTail, T, Error> TrySetNestedColumns<Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: PrepareColumn<CHead> + PrepareColumns<Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn try_set_nested_columns(
        &mut self,
        (head, tail): <(CHead, CTail) as TypedNestedTuple>::NestedTupleColumnType,
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared = <T as PrepareColumns<Error, (CHead, CTail)>>::prepare_columns(
            self,
            (head, tail),
            &context,
        )
        .map_err(|(_, error)| error)?;
        <T as PrepareColumns<Error, (CHead, CTail)>>::apply_columns(self, prepared);
        Ok(self)
    }
}
