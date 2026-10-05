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
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{DynColumn, TryGetDynamicColumns, builder_error::DynamicColumnError};
    /// use schema::{User, posts, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let ada: User = users::table.find(1).first(&mut conn)?;
    /// let grace: User = users::table.find(2).first(&mut conn)?;
    /// let ada_ref = &ada;
    /// let grace_ref = &grace;
    ///
    /// let (name, (age,)) = <&User as TryGetDynamicColumns>::try_get_dynamic_columns_ref(
    ///     &ada_ref,
    ///     (DynColumn::from(users::name), (DynColumn::from(users::age),)),
    /// )?;
    /// assert_eq!(name.map(String::as_str), Some("Ada"));
    /// assert_eq!(age, Some(&20));
    ///
    /// let (nickname,) = <&User as TryGetDynamicColumns>::try_get_dynamic_columns_ref(
    ///     &grace_ref,
    ///     (DynColumn::from(users::nickname),),
    /// )?;
    /// assert_eq!(nickname, None);
    ///
    /// let foreign = <&User as TryGetDynamicColumns>::try_get_dynamic_columns_ref(
    ///     &ada_ref,
    ///     (DynColumn::from(posts::title),),
    /// );
    /// assert!(matches!(foreign, Err(DynamicColumnError::UnknownColumn { .. })));
    /// # Ok(())
    /// # }
    /// ```
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
