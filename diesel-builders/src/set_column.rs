//! Submodule providing the `SetColumn` trait.

use diesel::Table;
use tuplities::prelude::NestTuple;

use crate::{
    AncestorOfIndex, BuildableTable, ColumnTyped, DescendantWithSelf, DynColumn, HasTableExt,
    NestedTables, OptionalRef, TableBuilder, TableExt, TypedColumn, ValueTyped,
    builder_error::DynamicSetColumnError,
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
    type Error: core::error::Error + Send + Sync + 'static;

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
pub trait TrySetDynamicColumn: Sized {
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
    fn try_set_dynamic_column_ref<VT: Clone + 'static>(
        &mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<&mut Self, DynamicSetColumnError>;

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
    fn try_set_dynamic_column<VT: Clone + 'static>(
        mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<Self, DynamicSetColumnError> {
        self.try_set_dynamic_column_ref(column, value)?;
        Ok(self)
    }
}

impl<Head> TrySetDynamicColumn for (Head,)
where
    Head: HasTableExt,
    Self: sealed::VariadicTrySetDynamicColumn<
            <<Head::Table as Table>::AllColumns as NestTuple>::Nested,
        >,
{
    #[inline]
    fn try_set_dynamic_column_ref<VT: Clone + 'static>(
        &mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<&mut Self, DynamicSetColumnError> {
        use sealed::VariadicTrySetDynamicColumn;
        self.variadic_try_set_dynamic_column(column, value)
    }
}

impl<Head, Tail> TrySetDynamicColumn for (Head, Tail)
where
    Head: HasTableExt + sealed::VariadicTrySetDynamicColumn<<Head::Table as TableExt>::NewRecord>,
    Tail: TrySetDynamicColumn,
{
    #[inline]
    fn try_set_dynamic_column_ref<VT: Clone + 'static>(
        &mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<&mut Self, DynamicSetColumnError> {
        if let Err(DynamicSetColumnError::UnknownColumn { .. }) =
            self.0.variadic_try_set_dynamic_column(column, value)
        {
            <Tail as TrySetDynamicColumn>::try_set_dynamic_column_ref(&mut self.1, column, value)?;
        }
        Ok(self)
    }
}

impl<T> TrySetDynamicColumn for TableBuilder<T>
where
    Self: sealed::VariadicTrySetDynamicColumn<
        <<T as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::ChainedNestedRecords,
    >,
    T: AncestorOfIndex<T> + BuildableTable
{
    #[inline]
    fn try_set_dynamic_column_ref<VT: Clone + 'static>(
        &mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<&mut Self, DynamicSetColumnError> {
        use sealed::VariadicTrySetDynamicColumn;
        self.variadic_try_set_dynamic_column(column, value)
    }
}

/// Sealed module for private traits.
mod sealed {
    use super::{DynColumn, DynamicSetColumnError, TableExt, TrySetColumn, TypedColumn};
    use crate::NestedColumns;

    /// Trait attempting to set a dynamic [`DynTypedColumn`], which may fail.
    pub trait VariadicTrySetDynamicColumn<Columns: NestedColumns> {
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
        fn variadic_try_set_dynamic_column<VT: Clone + 'static>(
            &mut self,
            column: DynColumn<VT>,
            value: &VT,
        ) -> Result<&mut Self, DynamicSetColumnError>;
    }

    impl<M, CHead> VariadicTrySetDynamicColumn<(CHead,)> for M
    where
        M: TrySetColumn<CHead>,
        CHead: TypedColumn<Table: TableExt>,
    {
        #[inline]
        fn variadic_try_set_dynamic_column<VT: Clone + 'static>(
            &mut self,
            column: DynColumn<VT>,
            value: &VT,
        ) -> Result<&mut Self, DynamicSetColumnError> {
            let value_any: &dyn core::any::Any = value;
            if column.column_name() == CHead::NAME
                && column.table_name() == <CHead::Table as TableExt>::TABLE_NAME
                && let Some(value) = value_any.downcast_ref::<CHead::ValueType>()
            {
                Ok(<Self as TrySetColumn<CHead>>::try_set_column(self, value.clone())
                    .map_err(|e| DynamicSetColumnError::Validation(Box::new(e)))?)
            } else {
                Err(DynamicSetColumnError::UnknownColumn {
                    table_name: column.table_name(),
                    column_name: column.column_name(),
                })
            }
        }
    }

    impl<M, CHead, CTail> VariadicTrySetDynamicColumn<(CHead, CTail)> for M
    where
        M: TrySetColumn<CHead> + VariadicTrySetDynamicColumn<CTail>,
        CHead: TypedColumn<Table: TableExt>,
        CTail: NestedColumns,
        (CHead, CTail): NestedColumns,
    {
        #[inline]
        fn variadic_try_set_dynamic_column<VT: Clone + 'static>(
            &mut self,
            column: DynColumn<VT>,
            value: &VT,
        ) -> Result<&mut Self, DynamicSetColumnError> {
            if column.column_name() == CHead::NAME
                && column.table_name() == <CHead::Table as TableExt>::TABLE_NAME
            {
                let value_any: &dyn core::any::Any = value;
                if let Some(value) = value_any.downcast_ref::<CHead::ValueType>() {
                    self.try_set_column(value.clone())
                        .map_err(|e| DynamicSetColumnError::Validation(Box::new(e)))
                } else {
                    <Self as VariadicTrySetDynamicColumn<CTail>>::variadic_try_set_dynamic_column(
                        self, column, value,
                    )
                }
            } else {
                <Self as VariadicTrySetDynamicColumn<CTail>>::variadic_try_set_dynamic_column(
                    self, column, value,
                )
            }
        }
    }
}
