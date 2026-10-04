//! Submodule providing the `TryMaySetColumns` trait.

use tuplities::prelude::IntoNestedTupleOption;

use crate::{
    TableExt, TypedColumn, ValidateColumn,
    columns::NestedColumns,
    mutation::{MutationContext, PrepareColumn, PrepareOptionalColumns},
};

/// Trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetNestedColumns<Error, CS: NestedColumns> {
    /// Attempt to set the `nested_values` of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{MayGetColumn, TryMaySetNestedColumns};
    /// use schema::{ValidationError, users};
    ///
    /// let mut values = user_values("Ada", 20, None);
    /// TryMaySetNestedColumns::<ValidationError, (users::nickname,)>::try_may_set_nested_columns(
    ///     &mut values,
    ///     (Some(Some("Ace".to_string())),),
    /// )?;
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string()))
    /// );
    /// assert_eq!(
    ///     TryMaySetNestedColumns::<ValidationError, (users::nickname,)>::try_may_set_nested_columns(
    ///         &mut values,
    ///         (Some(Some(String::new())),),
    ///     )
    ///     .err(),
    ///     Some(ValidationError::EmptyNickname)
    /// );
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string()))
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn try_may_set_nested_columns(
        &mut self,
        nested_values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<&mut Self, Error>;
}

impl<T, Error> TryMaySetNestedColumns<Error, ()> for T {
    #[inline]
    fn try_may_set_nested_columns(&mut self, _nested_values: ()) -> Result<&mut Self, Error> {
        Ok(self)
    }
}

impl<C1, T, Error> TryMaySetNestedColumns<Error, (C1,)> for T
where
    T: PrepareColumn<C1>,
    C1: TypedColumn<Table: TableExt>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
{
    #[inline]
    fn try_may_set_nested_columns(
        &mut self,
        nested_values: (Option<C1::ColumnType>,),
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared = <T as PrepareOptionalColumns<Error, (C1,)>>::prepare_optional_columns(
            self,
            nested_values,
            &context,
        )
        .map_err(|(_, error)| error)?;
        <T as PrepareOptionalColumns<Error, (C1,)>>::apply_optional_columns(self, prepared);
        Ok(self)
    }
}

impl<CHead, CTail, T, Error> TryMaySetNestedColumns<Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: PrepareColumn<CHead> + PrepareOptionalColumns<Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn try_may_set_nested_columns(
        &mut self,
        (head, tail): (
            Option<CHead::ColumnType>,
            <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> Result<&mut Self, Error> {
        let context = MutationContext::default();
        let prepared =
            <T as PrepareOptionalColumns<Error, (CHead, CTail)>>::prepare_optional_columns(
                self,
                (head, tail),
                &context,
            )
            .map_err(|(_, error)| error)?;
        <T as PrepareOptionalColumns<Error, (CHead, CTail)>>::apply_optional_columns(
            self, prepared,
        );
        Ok(self)
    }
}
