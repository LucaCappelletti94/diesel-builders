//! Submodule providing the `SetColumn` trait.

use diesel::Column;
use tuplities::prelude::NestedTuplePopFront;

use crate::{
    ColumnTyped, DynTypedColumn, NestedColumnsByValueType, OptionalRef, TableExt, TypedColumn,
    ValueTyped, builder_error::DynamicSetColumnError,
};

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn> {
    /// Set the value of the specified column.
    fn set_column(&mut self, value: impl Into<Column::ColumnType>) -> &mut Self;
}

/// Trait providing a failable setter for a specific Diesel column.
///
/// Extends [`SetColumn`].
pub trait MaySetColumn<Column: TypedColumn>: SetColumn<Column> {
    #[inline]
    /// Set the value of the specified column if the value is present.
    fn may_set_column(&mut self, value: Option<Column::ColumnType>) -> &mut Self {
        if let Some(v) = value {
            <Self as SetColumn<Column>>::set_column(self, v);
        }
        self
    }
}

impl<T, Column> MaySetColumn<Column> for T
where
    T: SetColumn<Column>,
    Column: TypedColumn,
{
}

/// Trait validating a specific Diesel column.
pub trait ValidateColumn<C: ValueTyped> {
    /// The associated error type for the operation.
    type Error;

    #[inline]
    /// Validate the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value is invalid.
    fn validate_column(_value: &C::ValueType) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    /// Validate the value of the specified column, given the context of the
    /// entire new record being built.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value is invalid.
    fn validate_column_in_context(&self, value: &C::ValueType) -> Result<(), Self::Error> {
        Self::validate_column(value)
    }
}

/// Trait attempting to set a specific Diesel column, which may fail.
///
/// Extends [`ValidateColumn`].
pub trait TrySetColumn<C: ColumnTyped>: ValidateColumn<C> {
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column(&mut self, value: impl Into<C::ColumnType>)
    -> Result<&mut Self, Self::Error>;
}

impl<T, C> TrySetColumn<C> for (T,)
where
    Self: SetColumn<C> + ValidateColumn<C>,
    C: TypedColumn,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let value = value.into();
        if let Some(value_ref) = value.as_optional_ref() {
            <Self as ValidateColumn<C>>::validate_column_in_context(self, value_ref)?;
        }
        <Self as SetColumn<C>>::set_column(self, value);
        Ok(self)
    }
}

impl<Head, Tail, C> TrySetColumn<C> for (Head, Tail)
where
    Self: SetColumn<C> + ValidateColumn<C>,
    C: TypedColumn,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let value = value.into();
        if let Some(value_ref) = value.as_optional_ref() {
            <Self as ValidateColumn<C>>::validate_column_in_context(self, value_ref)?;
        }
        <Self as SetColumn<C>>::set_column(self, value);
        Ok(self)
    }
}

/// Extension trait for [`SetColumn`] that allows specifying the column at the
/// method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait SetColumnExt: Sized {
    #[inline]
    /// Set the value of the specified column.
    fn set_column_ref<Column>(&mut self, value: impl Into<Column::ColumnType>) -> &mut Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(self, value)
    }

    #[inline]
    #[must_use]
    /// Set the value of the specified column.
    fn set_column<Column>(mut self, value: impl Into<Column::ColumnType>) -> Self
    where
        Column: TypedColumn,
        Self: SetColumn<Column>,
    {
        <Self as SetColumn<Column>>::set_column(&mut self, value);
        self
    }
}

impl<T> SetColumnExt for T {}

/// Extension trait for [`TrySetColumn`] that allows specifying the column at
/// the method level.
///
/// This trait provides a cleaner API where the column marker is specified as a
/// type parameter on the method rather than on the trait itself.
pub trait TrySetColumnExt: Sized {
    #[inline]
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column_ref<Column>(
        &mut self,
        value: impl Into<Column::ColumnType>,
    ) -> Result<&mut Self, <Self as ValidateColumn<Column>>::Error>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(self, value)
    }

    #[inline]
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_column<Column>(
        mut self,
        value: impl Into<Column::ColumnType>,
    ) -> Result<Self, <Self as ValidateColumn<Column>>::Error>
    where
        Column: TypedColumn,
        Self: TrySetColumn<Column>,
    {
        <Self as TrySetColumn<Column>>::try_set_column(&mut self, value)?;
        Ok(self)
    }
}

impl<T> TrySetColumnExt for T {}

/// Trait attempting to set a dynamic [`DynTypedColumn`], which may fail.
pub trait TrySetDynamicColumn<T: TableExt, ValueType> {
    /// Attempt to set the value of the specified dynamic column.
    ///
    /// # Arguments
    ///
    /// * `column` - The dynamic column to set.
    /// * `value` - The value to set for the column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_dynamic_column(
        &mut self,
        column: impl AsRef<dyn DynTypedColumn<Table = T, ValueType = ValueType>>,
        value: impl Into<ValueType>,
    ) -> Result<&mut Self, DynamicSetColumnError<T::Error>>;
}

impl<M, T, ValueType> TrySetDynamicColumn<T, ValueType> for M
where
    T: TableExt
        + NestedColumnsByValueType<
            ValueType,
            NestedColumns: NestedTuplePopFront<
                Front: TypedColumn<Table = T, ValueType = ValueType>,
            >,
        >,
    M: VariadicTrySetDynamicColumn<T::NestedColumns>,
{
    #[inline]
    fn try_set_dynamic_column(
        &mut self,
        column: impl AsRef<dyn DynTypedColumn<Table = T, ValueType = ValueType>>,
        value: impl Into<ValueType>,
    ) -> Result<&mut Self, DynamicSetColumnError<T::Error>> {
        <Self as VariadicTrySetDynamicColumn<T::NestedColumns>>::variadic_try_set_dynamic_column(
            self, column, value,
        )
    }
}

/// Alternative version of [`TrySetDynamicColumn`], with the trait generics
/// moved to the method level.
pub trait TrySetDynamicColumnExt: Sized {
    /// Attempt to set the value of the specified dynamic column.
    ///
    /// # Arguments
    ///
    /// * `column` - The dynamic column to set.
    /// * `value` - The value to set for the column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_dynamic_column_ref<T: TableExt, ValueType>(
        &mut self,
        column: impl AsRef<dyn DynTypedColumn<Table = T, ValueType = ValueType>>,
        value: impl Into<ValueType>,
    ) -> Result<&mut Self, DynamicSetColumnError<T::Error>>
    where
        Self: TrySetDynamicColumn<T, ValueType>,
    {
        <Self as TrySetDynamicColumn<T, ValueType>>::try_set_dynamic_column(self, column, value)
    }

    /// Attempt to set the value of the specified dynamic column, taking
    /// ownership of and returning self, so it can be chained.
    ///
    /// # Arguments
    ///
    /// * `column` - The dynamic column to set.
    /// * `value` - The value to set for the column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn try_set_dynamic_column<T: TableExt, ValueType>(
        mut self,
        column: impl AsRef<dyn DynTypedColumn<Table = T, ValueType = ValueType>>,
        value: impl Into<ValueType>,
    ) -> Result<Self, DynamicSetColumnError<T::Error>>
    where
        Self: TrySetDynamicColumn<T, ValueType>,
    {
        Self::try_set_dynamic_column_ref(&mut self, column, value)?;
        Ok(self)
    }
}

impl<M> TrySetDynamicColumnExt for M {}

/// Trait attempting to set a dynamic [`DynTypedColumn`], which may fail.
pub trait VariadicTrySetDynamicColumn<Columns>
where
    Columns: NestedTuplePopFront<Front: TypedColumn<Table: TableExt>>,
{
    /// Attempt to set the value of the specified dynamic column.
    ///
    /// # Arguments
    ///
    /// * `column` - The dynamic column to set.
    /// * `value` - The value to set for the column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    fn variadic_try_set_dynamic_column(
        &mut self,
        column: impl AsRef<
            dyn DynTypedColumn<
                    Table = <Columns::Front as Column>::Table,
                    ValueType = <Columns::Front as ValueTyped>::ValueType,
                >,
        >,
        value: impl Into<<Columns::Front as ValueTyped>::ValueType>,
    ) -> Result<
        &mut Self,
        DynamicSetColumnError<<<Columns::Front as Column>::Table as TableExt>::Error>,
    >;
}

impl<M, Head> VariadicTrySetDynamicColumn<(Head,)> for M
where
    M: TrySetColumn<Head>,
    Head: TypedColumn<Table: TableExt + NestedColumnsByValueType<<Head as ValueTyped>::ValueType>>,
    <<Head as Column>::Table as TableExt>::Error: From<<M as ValidateColumn<Head>>::Error>,
{
    #[inline]
    fn variadic_try_set_dynamic_column(
        &mut self,
        column: impl AsRef<
            dyn DynTypedColumn<
                    Table = <Head as Column>::Table,
                    ValueType = <Head as ValueTyped>::ValueType,
                >,
        >,
        value: impl Into<<Head as ValueTyped>::ValueType>,
    ) -> Result<&mut Self, DynamicSetColumnError<<<Head as Column>::Table as TableExt>::Error>>
    {
        let column = column.as_ref();
        if column.column_name() == Head::NAME {
            <Self as TrySetColumn<Head>>::try_set_column(self, value.into())
                .map_err(|e| DynamicSetColumnError::Validation(e.into()))
        } else {
            Err(DynamicSetColumnError::UnknownColumn(column.column_name()))
        }
    }
}

impl<M, Head, Tail> VariadicTrySetDynamicColumn<(Head, Tail)> for M
where
    M: TrySetColumn<Head> + VariadicTrySetDynamicColumn<Tail>,
    Tail: NestedTuplePopFront<Front: TypedColumn<Table = Head::Table, ValueType = Head::ValueType>>,
    Head: TypedColumn<Table: TableExt + NestedColumnsByValueType<<Head as ValueTyped>::ValueType>>,
    <<Head as Column>::Table as TableExt>::Error: From<<M as ValidateColumn<Head>>::Error>,
{
    #[inline]
    fn variadic_try_set_dynamic_column(
        &mut self,
        column: impl AsRef<
            dyn DynTypedColumn<
                    Table = <Head as Column>::Table,
                    ValueType = <Head as ValueTyped>::ValueType,
                >,
        >,
        value: impl Into<<Head as ValueTyped>::ValueType>,
    ) -> Result<&mut Self, DynamicSetColumnError<<<Head as Column>::Table as TableExt>::Error>>
    {
        let column_ref = column.as_ref();
        if column_ref.column_name() == Head::NAME {
            <Self as TrySetColumn<Head>>::try_set_column(self, value.into())
                .map_err(|e| DynamicSetColumnError::Validation(e.into()))
        } else {
            <Self as VariadicTrySetDynamicColumn<Tail>>::variadic_try_set_dynamic_column(
                self, column, value,
            )
        }
    }
}
