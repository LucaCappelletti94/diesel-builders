//! Submodule providing the `TryGetDynamicColumn` trait.

use diesel::Table;
use sealed::VariadicTryGetDynamicColumn;
use tuplities::prelude::NestTuple;

use crate::{DynColumn, HasTableExt, TableExt, builder_error::DynamicColumnError};

/// Trait attempting to get a dynamic [`DynColumn`], which may fail.
pub trait TryGetDynamicColumn {
    /// Attempt to get the value of the specified dynamic column.
    ///
    /// # Arguments
    ///
    /// * `column` - The dynamic column to get.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be retrieved.
    fn try_get_dynamic_column_ref<VT: 'static>(
        &self,
        column: DynColumn<VT>,
    ) -> Result<Option<&VT>, DynamicColumnError>;

    /// Attempt to get the value of the specified dynamic column.
    ///
    /// # Arguments
    ///
    /// * `column` - The dynamic column to get.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be retrieved.
    fn try_get_dynamic_column<VT: Clone + 'static>(
        &self,
        column: DynColumn<VT>,
    ) -> Result<Option<VT>, DynamicColumnError> {
        self.try_get_dynamic_column_ref(column).map(|opt| opt.cloned())
    }
}

macro_rules! impl_try_get_dynamic_column {
    ($($T:ty),*) => {
        $(
            impl<Head> TryGetDynamicColumn for $T
            where
                Head: HasTableExt,
                Self: sealed::VariadicTryGetDynamicColumn<
                    <<Head::Table as Table>::AllColumns as NestTuple>::Nested,
                >,
            {
                #[inline]
                fn try_get_dynamic_column_ref<VT: 'static>(
                    &self,
                    column: DynColumn<VT>,
                ) -> Result<Option<&VT>, DynamicColumnError> {
                    self.variadic_try_get_dynamic_column(column)
                }
            }
        )*
    };
}

impl_try_get_dynamic_column!(&Head, Box<Head>, std::rc::Rc<Head>, std::sync::Arc<Head>);

impl<Head> TryGetDynamicColumn for (Head,)
where
    Head: HasTableExt,
    Self: sealed::VariadicTryGetDynamicColumn<
            <<Head::Table as Table>::AllColumns as NestTuple>::Nested,
        >,
{
    #[inline]
    fn try_get_dynamic_column_ref<VT: 'static>(
        &self,
        column: DynColumn<VT>,
    ) -> Result<Option<&VT>, DynamicColumnError> {
        self.variadic_try_get_dynamic_column(column)
    }
}

impl<Head, Tail> TryGetDynamicColumn for (Head, Tail)
where
    Head: HasTableExt + sealed::VariadicTryGetDynamicColumn<<Head::Table as TableExt>::NewRecord>,
    Tail: TryGetDynamicColumn,
{
    #[inline]
    fn try_get_dynamic_column_ref<VT: 'static>(
        &self,
        column: DynColumn<VT>,
    ) -> Result<Option<&VT>, DynamicColumnError> {
        match self.0.variadic_try_get_dynamic_column(column) {
            Err(DynamicColumnError::UnknownColumn { .. }) => {
                self.1.try_get_dynamic_column_ref(column)
            }
            res => res,
        }
    }
}

/// Local module for sealed trait.
mod sealed {
    use super::super::GetColumn;
    use crate::{
        DynColumn, NestedColumns, OptionalRef, TableExt, TypedColumn,
        builder_error::DynamicColumnError,
    };

    /// Trait for variadic dynamic column retrieval.
    pub trait VariadicTryGetDynamicColumn<Columns: NestedColumns> {
        /// Try to get a dynamic column from a nested structure.
        fn variadic_try_get_dynamic_column<VT: 'static>(
            &self,
            column: DynColumn<VT>,
        ) -> Result<Option<&VT>, DynamicColumnError>;
    }

    impl<M, CHead> VariadicTryGetDynamicColumn<(CHead,)> for M
    where
        M: GetColumn<CHead>,
        CHead: TypedColumn<Table: TableExt> + 'static,
        CHead::ColumnType: 'static,
    {
        #[inline]
        fn variadic_try_get_dynamic_column<VT: 'static>(
            &self,
            column: DynColumn<VT>,
        ) -> Result<Option<&VT>, DynamicColumnError> {
            if column.column_name() == CHead::NAME
                && column.table_name() == <CHead::Table as TableExt>::TABLE_NAME
                && core::any::TypeId::of::<VT>() == core::any::TypeId::of::<CHead::ValueType>()
            {
                let Some(value) = self.get_column_ref().as_optional_ref() else {
                    return Ok(None);
                };

                let value_any: &dyn core::any::Any = value;
                if let Some(value) = value_any.downcast_ref::<VT>() {
                    return Ok(Some(value));
                }
            }
            Err(DynamicColumnError::UnknownColumn {
                table_name: column.table_name(),
                column_name: column.column_name(),
            })
        }
    }

    impl<M, CHead, CTail> VariadicTryGetDynamicColumn<(CHead, CTail)> for M
    where
        M: GetColumn<CHead> + VariadicTryGetDynamicColumn<CTail>,
        CHead: TypedColumn<Table: TableExt> + 'static,
        CHead::ColumnType: 'static,
        CTail: NestedColumns,
        (CHead, CTail): NestedColumns,
    {
        #[inline]
        fn variadic_try_get_dynamic_column<VT: 'static>(
            &self,
            column: DynColumn<VT>,
        ) -> Result<Option<&VT>, DynamicColumnError> {
            if column.column_name() == CHead::NAME
                && column.table_name() == <CHead::Table as TableExt>::TABLE_NAME
                && core::any::TypeId::of::<VT>() == core::any::TypeId::of::<CHead::ValueType>()
            {
                let Some(value) = self.get_column_ref().as_optional_ref() else {
                    return Ok(None);
                };

                let value_any: &dyn core::any::Any = value;
                if let Some(value) = value_any.downcast_ref::<VT>() {
                    return Ok(Some(value));
                }
                return Err(DynamicColumnError::UnknownColumn {
                    table_name: column.table_name(),
                    column_name: column.column_name(),
                });
            }

            <Self as VariadicTryGetDynamicColumn<CTail>>::variadic_try_get_dynamic_column(
                self, column,
            )
        }
    }
}
