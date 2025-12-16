//! Submodule providing the `TryMaySetColumns` trait.

use std::borrow::Borrow;

use tuplities::prelude::IntoNestedTupleOption;

use crate::{
    MayValidateNestedColumns, TrySetColumn, TypedColumn, ValidateColumn, columns::NestedColumns,
};

/// Trait indicating a builder which may try to set multiple columns.
pub trait TryMaySetNestedColumns<Error, CS: NestedColumns>:
    MayValidateNestedColumns<Error, CS>
{
    /// Attempt to set the `nested_values` of the specified columns.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be set.
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
    T: TrySetColumn<C1>,
    C1: TypedColumn<ValueType: Borrow<T::Borrowed>>,
    Error: From<<T as ValidateColumn<C1>>::Error>,
{
    #[inline]
    fn try_may_set_nested_columns(
        &mut self,
        nested_values: (Option<C1::ColumnType>,),
    ) -> Result<&mut Self, Error> {
        if let Some(value) = nested_values.0 {
            self.try_set_column(value)?;
        }
        Ok(self)
    }
}

impl<CHead, CTail, T, Error> TryMaySetNestedColumns<Error, (CHead, CTail)> for T
where
    CHead: TypedColumn<ValueType: Borrow<T::Borrowed>>,
    CTail: NestedColumns,
    (CHead, CTail):
        NestedColumns<NestedTupleColumnType = (CHead::ColumnType, CTail::NestedTupleColumnType)>,
    T: TrySetColumn<CHead> + TryMaySetNestedColumns<Error, CTail>,
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
        if let Some(value) = head {
            self.try_set_column(value)?;
        }
        self.try_may_set_nested_columns(tail)?;
        Ok(self)
    }
}
