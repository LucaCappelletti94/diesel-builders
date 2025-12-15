//! Trait for fallibly setting multiple nested columns.
use tuplities::prelude::IntoNestedTupleOption;

use crate::{TrySetColumn, TypedColumn, TypedNestedTuple, ValidateColumn, columns::NestedColumns};

/// Trait indicating a builder can validate multiple nested columns.
pub trait ValidateNestedColumns<Error, CS: NestedColumns> {
    /// Validate the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column fails validation.
    fn validate_nested_columns(&self, values: &CS::NestedTupleColumnType) -> Result<(), Error>;
}

impl<T, Error> ValidateNestedColumns<Error, ()> for T {
    #[inline]
    fn validate_nested_columns(&self, _values: &()) -> Result<(), Error> {
        Ok(())
    }
}

impl<C1, T, Error> ValidateNestedColumns<Error, (C1,)> for T
where
    Error: From<<T as ValidateColumn<C1>>::Error>,
    T: ValidateColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn validate_nested_columns(&self, values: &(C1::ColumnType,)) -> Result<(), Error> {
        Ok(self.validate_column_in_context(&values.0)?)
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
        self.validate_column_in_context(head)?;
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
    fn may_validate_nested_columns(
        &self,
        values: &<CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<(), Error>;
}

impl<T, Error> MayValidateNestedColumns<Error, ()> for T {
    #[inline]
    fn may_validate_nested_columns(&self, _values: &()) -> Result<(), Error> {
        Ok(())
    }
}

impl<C1, T, Error> MayValidateNestedColumns<Error, (C1,)> for T
where
    Error: From<<T as ValidateColumn<C1>>::Error>,
    T: ValidateColumn<C1>,
    C1: TypedColumn,
{
    #[inline]
    fn may_validate_nested_columns(&self, values: &(Option<C1::ColumnType>,)) -> Result<(), Error> {
        if let Some(ref v1) = values.0 {
            self.validate_column_in_context(v1)?;
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
        values: &(
            Option<CHead::ColumnType>,
            <CTail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> Result<(), Error> {
        if let Some(ref head) = values.0 {
            self.validate_column_in_context(head)?;
        }
        self.may_validate_nested_columns(&values.1)?;
        Ok(())
    }
}

/// Trait indicating a builder can fallibly set multiple columns.
pub trait TrySetNestedColumns<Error, CS: NestedColumns>: ValidateNestedColumns<Error, CS> {
    /// Attempt to set the values of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
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
    T: TrySetColumn<C1>,
    C1: TypedColumn,
    Error: From<<T as ValidateColumn<C1>>::Error>,
{
    #[inline]
    fn try_set_nested_columns(&mut self, values: (C1::ColumnType,)) -> Result<&mut Self, Error> {
        self.try_set_column(values.0)?;
        Ok(self)
    }
}

impl<CHead, CTail, T, Error> TrySetNestedColumns<Error, (CHead, CTail)> for T
where
    CHead: TypedColumn,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: TrySetColumn<CHead> + TrySetNestedColumns<Error, CTail>,
    Error: From<<T as ValidateColumn<CHead>>::Error>,
{
    #[inline]
    fn try_set_nested_columns(
        &mut self,
        (head, tail): <(CHead, CTail) as TypedNestedTuple>::NestedTupleColumnType,
    ) -> Result<&mut Self, Error> {
        self.try_set_column(head)?;
        self.try_set_nested_columns(tail)?;
        Ok(self)
    }
}
