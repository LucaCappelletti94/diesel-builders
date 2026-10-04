//! Submodule providing the `SetColumn` trait.

use crate::{
    AncestorOfIndex, BuildableTable, ColumnTyped, DescendantWithSelf, DynColumn, NestedTables,
    OptionalRef, TableBuilder, TableExt, TypedColumn, ValueTyped,
    builder_error::DynamicColumnError,
};

/// Trait providing a setter for a specific Diesel column.
pub trait SetColumn<Column: TypedColumn> {
    /// Set the value of the specified column.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::SetColumn;
    /// use schema::users;
    ///
    /// let mut staged = users::table::empty_new_values();
    /// SetColumn::<users::name>::set_column(&mut staged, "Ada".to_owned());
    /// SetColumn::<users::age>::set_column(&mut staged, 20);
    /// SetColumn::<users::nickname>::set_column(&mut staged, None::<String>);
    ///
    /// assert_eq!(staged.may_get_column_ref::<users::name>().map(String::as_str), Some("Ada"));
    /// assert_eq!(staged.may_get_column_ref::<users::age>(), Some(&20));
    /// assert_eq!(staged.may_get_column_ref::<users::nickname>(), Some(&None));
    /// # }
    /// ```
    fn set_column(&mut self, value: impl Into<Column::ColumnType>) -> &mut Self;
}

/// Trait providing a failable setter for a specific Diesel column.
///
/// Extends [`SetColumn`].
pub trait MaySetColumn<Column: TypedColumn>: SetColumn<Column> {
    #[inline]
    /// Set the value of the specified column if the value is present.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::MaySetColumn;
    /// use schema::users;
    ///
    /// let mut staged = users::table::empty_new_values();
    /// MaySetColumn::<users::nickname>::may_set_column(&mut staged, Some(Some("Ace".to_owned())));
    /// assert_eq!(
    ///     staged.may_get_column_ref::<users::nickname>().and_then(Option::as_deref),
    ///     Some("Ace")
    /// );
    ///
    /// let mut untouched = users::table::empty_new_values();
    /// MaySetColumn::<users::age>::may_set_column(&mut untouched, None);
    /// assert_eq!(untouched.may_get_column_ref::<users::age>(), None);
    /// # }
    /// ```
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

/// Validates a column value independently of other fields.
pub trait ValidateColumn<C: ValueTyped> {
    /// The associated error type for the operation.
    type Error: core::error::Error + Send + Sync + 'static;

    #[inline]
    /// Validate the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column value is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{TableExt, ValidateColumn};
    /// use schema::{ValidationError, posts, users};
    ///
    /// type UserValues = <users::table as TableExt>::NewValues;
    /// assert_eq!(<UserValues as ValidateColumn<users::age>>::validate_column(&20), Ok(()));
    /// assert_eq!(
    ///     <UserValues as ValidateColumn<users::age>>::validate_column(&17),
    ///     Err(ValidationError::AgeTooYoung)
    /// );
    /// let nickname = "Ace".to_owned();
    /// assert_eq!(<UserValues as ValidateColumn<users::nickname>>::validate_column(&nickname), Ok(()));
    /// let empty = String::new();
    /// assert_eq!(
    ///     <UserValues as ValidateColumn<users::nickname>>::validate_column(&empty),
    ///     Err(ValidationError::EmptyNickname)
    /// );
    ///
    /// type PostValues = <posts::table as TableExt>::NewValues;
    /// let title = "First".to_owned();
    /// assert_eq!(<PostValues as ValidateColumn<posts::title>>::validate_column(&title), Ok(()));
    /// # }
    /// ```
    fn validate_column(_value: &C::ValueType) -> Result<(), Self::Error> {
        Ok(())
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
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::TrySetColumn;
    /// use schema::{ValidationError, users};
    ///
    /// let mut staged = users::table::empty_new_values();
    /// TrySetColumn::<users::name>::try_set_column(&mut staged, "Ada".to_owned())?;
    /// TrySetColumn::<users::age>::try_set_column(&mut staged, 20)?;
    /// assert_eq!(staged.may_get_column_ref::<users::age>(), Some(&20));
    ///
    /// let mut rejected = users::table::empty_new_values();
    /// TrySetColumn::<users::name>::try_set_column(&mut rejected, "Ada".to_owned())?;
    /// let err = TrySetColumn::<users::age>::try_set_column(&mut rejected, 17);
    /// assert_eq!(err.err(), Some(ValidationError::AgeTooYoung));
    /// assert_eq!(rejected.may_get_column_ref::<users::name>(), Some(&"Ada".to_owned()));
    /// assert_eq!(rejected.may_get_column_ref::<users::age>(), None);
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_column(&mut self, value: impl Into<C::ColumnType>)
    -> Result<&mut Self, Self::Error>;
}

/// Emits the identical leaf `TrySetColumn` body for each `NewValues` tuple
/// arity. These are non-recursive leaf impls (the derive emits per-column
/// `SetColumn`/`ValidateColumn` on `NewValues`), so both arities share one
/// body that validates an optional value then sets it.
macro_rules! impl_try_set_column_for_tuple {
    ($( impl[$($generics:ident),+] for $tuple:ty ),+ $(,)?) => {
        $(
            impl<$($generics),+, C> TrySetColumn<C> for $tuple
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
                        <Self as ValidateColumn<C>>::validate_column(value_ref)?;
                    }
                    <Self as SetColumn<C>>::set_column(self, value);
                    Ok(self)
                }
            }
        )+
    };
}

impl_try_set_column_for_tuple! {
    impl[T] for (T,),
    impl[Head, Tail] for (Head, Tail),
}

/// Extension trait for [`SetColumn`] that allows specifying the column at the
/// method level.
pub trait SetColumnExt: Sized {
    #[inline]
    /// Set the value of the specified column.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use schema::users;
    ///
    /// let mut staged = users::table::empty_new_values();
    /// staged
    ///     .set_column_ref::<users::name>("Ada".to_owned())
    ///     .set_column_ref::<users::age>(20)
    ///     .set_column_ref::<users::nickname>("Ace".to_owned());
    ///
    /// assert_eq!(staged.may_get_column_ref::<users::name>().map(String::as_str), Some("Ada"));
    /// assert_eq!(staged.may_get_column_ref::<users::age>(), Some(&20));
    /// assert_eq!(
    ///     staged.may_get_column_ref::<users::nickname>().map(Option::as_deref),
    ///     Some(Some("Ace"))
    /// );
    /// # }
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use schema::users;
    ///
    /// let staged = users::table::empty_new_values()
    ///     .set_column::<users::name>("Ada".to_owned())
    ///     .set_column::<users::age>(20)
    ///     .set_column::<users::nickname>(None::<String>);
    /// assert_eq!(staged.may_get_column_ref::<users::nickname>(), Some(&None));
    ///
    /// let name_only = users::table::empty_new_values().set_column::<users::name>("Grace".to_owned());
    /// assert_eq!(name_only.may_get_column_ref::<users::nickname>(), Some(&None));
    /// # }
    /// ```
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
pub trait TrySetColumnExt: Sized {
    #[inline]
    /// Attempt to set the value of the specified column.
    ///
    /// # Errors
    ///
    /// Returns an error if the column cannot be set.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{ValidationError, users};
    ///
    /// let mut staged = users::table::empty_new_values();
    /// staged.try_set_column_ref::<users::name>("Ada".to_owned())?;
    /// staged.try_set_column_ref::<users::age>(20)?;
    ///
    /// let err = staged.try_set_column_ref::<users::nickname>("".to_owned());
    /// assert_eq!(err.err(), Some(ValidationError::EmptyNickname));
    /// assert_eq!(staged.may_get_column_ref::<users::name>().map(String::as_str), Some("Ada"));
    /// assert_eq!(staged.may_get_column_ref::<users::age>(), Some(&20));
    /// assert_eq!(staged.may_get_column_ref::<users::nickname>(), Some(&None));
    /// # Ok(())
    /// # }
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{ValidationError, users};
    ///
    /// let staged = users::table::empty_new_values()
    ///     .try_set_column::<users::name>("Ada".to_owned())?
    ///     .try_set_column::<users::age>(20)?;
    /// assert_eq!(staged.may_get_column_ref::<users::name>().map(String::as_str), Some("Ada"));
    /// assert_eq!(staged.may_get_column_ref::<users::age>(), Some(&20));
    ///
    /// let err = users::table::empty_new_values().try_set_column::<users::age>(17);
    /// assert_eq!(err.err(), Some(ValidationError::AgeTooYoung));
    /// # Ok(())
    /// # }
    /// ```
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

/// Trait attempting to set a dynamic [`DynColumn`], which may fail.
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
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::builder_error::DynamicColumnError;
    /// use diesel_builders::{DynColumn, TrySetDynamicColumn};
    /// use schema::{ValidationError, posts, users};
    ///
    /// let mut builder = users::table::try_builder()?;
    /// let name = "Ada".to_owned();
    /// builder.try_set_dynamic_column_ref::<String>(DynColumn::from(users::name), &name)?;
    /// assert_eq!(builder.may_get_column_ref::<users::name>(), Some(&"Ada".to_owned()));
    ///
    /// let rejected = builder.try_set_dynamic_column_ref::<i32>(DynColumn::from(users::age), &17);
    /// assert!(matches!(&rejected, Err(DynamicColumnError::Validation(error))
    ///     if error.downcast_ref::<ValidationError>() == Some(&ValidationError::AgeTooYoung)));
    ///
    /// let foreign = builder.try_set_dynamic_column_ref::<String>(DynColumn::from(posts::title), &name);
    /// assert!(matches!(foreign, Err(DynamicColumnError::UnknownColumn { table_name, column_name })
    ///     if (table_name, column_name) == ("posts", "title")));
    ///
    /// assert_eq!(builder.may_get_column_ref::<users::age>(), Some(&18));
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_dynamic_column_ref<VT: Clone + 'static>(
        &mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<&mut Self, DynamicColumnError>;

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
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::DynColumn;
    /// use schema::users;
    ///
    /// let title = "Ada".to_owned();
    /// let age = 20;
    /// let builder = users::table::try_builder()?
    ///     .try_set_dynamic_column::<String>(DynColumn::from(users::name), &title)?
    ///     .try_set_dynamic_column::<i32>(DynColumn::from(users::age), &age)?;
    /// assert_eq!(builder.may_get_column_ref::<users::name>(), Some(&"Ada".to_owned()));
    /// assert_eq!(builder.may_get_column_ref::<users::age>(), Some(&20));
    /// # Ok(())
    /// # }
    /// ```
    fn try_set_dynamic_column<VT: Clone + 'static>(
        mut self,
        column: DynColumn<VT>,
        value: &VT,
    ) -> Result<Self, DynamicColumnError> {
        self.try_set_dynamic_column_ref(column, value)?;
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
    ) -> Result<&mut Self, DynamicColumnError> {
        use sealed::VariadicTrySetDynamicColumn;
        self.variadic_try_set_dynamic_column(column, value)
    }
}

/// Sealed module for private traits.
mod sealed {
    use super::{DynColumn, DynamicColumnError, TableExt, TrySetColumn, TypedColumn};
    use crate::NestedColumns;

    /// Trait attempting to set a dynamic [`DynColumn`], which may fail.
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
        ) -> Result<&mut Self, DynamicColumnError>;
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
        ) -> Result<&mut Self, DynamicColumnError> {
            let value_any: &dyn core::any::Any = value;
            if column.column_name() == CHead::NAME
                && column.table_name() == <CHead::Table as TableExt>::TABLE_NAME
                && let Some(value) = value_any.downcast_ref::<CHead::ValueType>()
            {
                Ok(<Self as TrySetColumn<CHead>>::try_set_column(self, value.clone())
                    .map_err(|e| DynamicColumnError::Validation(Box::new(e)))?)
            } else {
                Err(DynamicColumnError::UnknownColumn {
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
        ) -> Result<&mut Self, DynamicColumnError> {
            if column.column_name() == CHead::NAME
                && column.table_name() == <CHead::Table as TableExt>::TABLE_NAME
            {
                let value_any: &dyn core::any::Any = value;
                if let Some(value) = value_any.downcast_ref::<CHead::ValueType>() {
                    return self
                        .try_set_column(value.clone())
                        .map_err(|e| DynamicColumnError::Validation(Box::new(e)));
                }
            }

            <Self as VariadicTrySetDynamicColumn<CTail>>::variadic_try_set_dynamic_column(
                self, column, value,
            )
        }
    }
}
