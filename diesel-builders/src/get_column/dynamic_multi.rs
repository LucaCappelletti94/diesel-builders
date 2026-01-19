//! Submodule providing the `TryGetDynamicColumns` trait for variadic dynamic
//! column retrieval.

use tuplities::prelude::{FlattenNestedTuple, IntoNestedTupleOption, NestedTupleRef};

use crate::{
    DynColumn, TypedNestedTuple, builder_error::DynamicColumnError,
    get_column::dynamic::TryGetDynamicColumn,
};

/// Trait attempting to get multiple dynamic [`DynColumn`]s, which may fail.
pub trait TryGetDynamicColumns {
    /// Attempt to get the value of the specified dynamic columns.
    ///
    /// # Arguments
    ///
    /// * `columns` - The dynamic columns to get.
    ///
    /// # Errors
    ///
    /// Returns an error if any column cannot be retrieved (e.g., unknown
    /// column).
    fn try_get_dynamic_columns_ref<'a, DCS>(
        &'a self,
        columns: DCS,
    ) -> Result<
        <<<DCS as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions,
        DynamicColumnError,
    >
    where
        DCS: TypedNestedTuple + sealed::VariadicTryGetDynamicColumns<'a, Self>;
}

impl<T> TryGetDynamicColumns for T {
    fn try_get_dynamic_columns_ref<'a, DCS>(
        &'a self,
        columns: DCS,
    ) -> Result<<<<DCS as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions, DynamicColumnError>
    where
        DCS: TypedNestedTuple + sealed::VariadicTryGetDynamicColumns<'a, Self>,
    {
        columns.variadic_try_get_dynamic_columns(self)
    }
}

/// Sealed trait module for internal variadic logic.
pub(crate) mod sealed {
    use super::{
        DynColumn, DynamicColumnError, FlattenNestedTuple, IntoNestedTupleOption, NestedTupleRef,
        TryGetDynamicColumn, TypedNestedTuple,
    };

    /// Trait for retrieving dynamic columns from a variadic tuple of columns.
    pub trait VariadicTryGetDynamicColumns<'a, T: ?Sized>: TypedNestedTuple {
        /// Recursively retrieves dynamic columns.
        fn variadic_try_get_dynamic_columns(
            self,
            target: &'a T,
        ) -> Result<<<<Self as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions, DynamicColumnError>;
    }

    impl<'a, T> VariadicTryGetDynamicColumns<'a, T> for () {
        fn variadic_try_get_dynamic_columns(
            self,
            _target: &'a T,
        ) -> Result<(), DynamicColumnError> {
            Ok(())
        }
    }

    impl<'a, Head, Tail, T> VariadicTryGetDynamicColumns<'a, T> for (DynColumn<Head>, Tail)
    where
        Head: 'static + std::fmt::Debug + Clone,
        T: TryGetDynamicColumn,
        Tail: VariadicTryGetDynamicColumns<'a, T>,
        (DynColumn<Head>, Tail): TypedNestedTuple<NestedTupleValueType = (Head, Tail::NestedTupleValueType)>
            + FlattenNestedTuple,
    {
        fn variadic_try_get_dynamic_columns(
            self,
            target: &'a T,
        ) -> Result<<<<Self as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions, DynamicColumnError>{
            let (head, tail) = (self.0, self.1);
            let head_res = target.try_get_dynamic_column_ref(head)?;
            let tail_res = tail.variadic_try_get_dynamic_columns(target)?;
            Ok((head_res, tail_res))
        }
    }

    impl<'a, Head, T> VariadicTryGetDynamicColumns<'a, T> for (DynColumn<Head>,)
    where
        Head: 'static + std::fmt::Debug + Clone,
        T: TryGetDynamicColumn,
        (DynColumn<Head>,): TypedNestedTuple<NestedTupleValueType = (Head,)> + FlattenNestedTuple,
    {
        fn variadic_try_get_dynamic_columns(
            self,
            target: &'a T,
        ) -> Result<(Option<&'a Head>,), DynamicColumnError> {
            let head = self.0;
            let head_res = target.try_get_dynamic_column_ref(head)?;
            Ok((head_res,))
        }
    }
}
