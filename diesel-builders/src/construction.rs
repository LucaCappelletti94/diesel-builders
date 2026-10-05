//! Staged, validated default construction for table builders.

use std::convert::Infallible;

use tuplities::prelude::NestedTupleIndexMut;

use crate::{
    AncestorOfIndex, BuildableTable, DescendantOf, EitherValidationError, MayGetColumn,
    NestedColumns, OptionalRef, SetColumn, SetHomogeneousNestedColumns, TableBuilder,
    TableBuilderBundle, TakeColumn, TypedColumn, ValidateColumn, VerticalSameAsGroup,
    builder_bundle::BundlableTableExt,
};

/// Staged default construction state.
pub trait BuildDefaults: Sized {
    /// The validation error of the staged default group.
    type Error: std::error::Error + Send + Sync + 'static;
    /// The checked bundles containing successfully validated values.
    type CheckedBundles;

    /// Validates and moves each final staged default once.
    ///
    /// # Errors
    ///
    /// Returns the first validation failure.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{BuildDefaults, DefaultBuilder, DefaultBundle, SetColumn};
    /// use schema::*;
    ///
    /// let mut staged = DefaultBuilder::<profiles::table>::new();
    /// assert!(staged.check_defaults().is_ok());
    ///
    /// let mut bundle = DefaultBundle::<users::table>::new();
    /// SetColumn::<users::age>::set_column(&mut bundle, 17);
    /// assert!(matches!(bundle.check_defaults(), Err(ValidationError::AgeTooYoung)));
    /// # }
    /// ```
    fn check_defaults(&mut self) -> Result<(), Self::Error>;

    /// Consumes the staged graph and returns its checked state.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{BuildDefaults, DefaultBundle, MayGetColumn, SetColumn};
    /// use schema::*;
    ///
    /// let mut staged = DefaultBundle::<users::table>::new();
    /// assert_eq!(staged.check_defaults(), Ok(()));
    /// let bundle = staged.into_checked();
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column_ref(&bundle), Some(&18));
    ///
    /// let mut staged = DefaultBundle::<users::table>::new();
    /// SetColumn::<users::age>::set_column(&mut staged, 17);
    /// assert_eq!(staged.check_defaults(), Err(ValidationError::AgeTooYoung));
    /// let bundle = staged.into_checked();
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column_ref(&bundle), None);
    /// # }
    /// ```
    #[must_use]
    fn into_checked(self) -> Self::CheckedBundles;
}

/// Keeps one table's raw defaults separate from its checked values.
pub struct DefaultBundle<S: BundlableTableExt> {
    /// Raw, unchecked staged values.
    raw: S::NewValues,
    /// Validated values moved out of the raw state.
    checked: S::NewValues,
}

impl<S: BundlableTableExt> DefaultBundle<S> {
    /// Stages declared defaults alongside checked empty values.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{BuildDefaults, DefaultBundle, MayGetColumn};
    /// use schema::*;
    ///
    /// let mut staged = DefaultBundle::<users::table>::new();
    /// assert_eq!(staged.check_defaults(), Ok(()));
    /// let bundle = staged.into_checked();
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column_ref(&bundle), Some(&18));
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column_ref(&bundle), None);
    /// let nickname = MayGetColumn::<users::nickname>::may_get_column_ref(&bundle);
    /// assert_eq!(nickname, Some(&None::<String>));
    /// # }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { raw: S::default_new_values(), checked: S::empty_new_values() }
    }

    /// Validates one present raw value and moves it into the checked state.
    fn check_and_move_one<C>(&mut self) -> Result<(), <S::NewValues as ValidateColumn<C>>::Error>
    where
        C: TypedColumn<Table = S>,
        S::NewValues: MayGetColumn<C> + ValidateColumn<C> + TakeColumn<C> + SetColumn<C>,
    {
        if let Some(value) = <S::NewValues as MayGetColumn<C>>::may_get_column_ref(&self.raw)
            && let Some(value) = value.as_optional_ref()
        {
            <S::NewValues as ValidateColumn<C>>::validate_column(value)?;
        }
        if let Some(value) = <S::NewValues as TakeColumn<C>>::take_column(&mut self.raw) {
            <S::NewValues as SetColumn<C>>::set_column(&mut self.checked, value);
        }
        Ok(())
    }
}

impl<S: BundlableTableExt> Default for DefaultBundle<S> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<S> BuildDefaults for DefaultBundle<S>
where
    S: BundlableTableExt,
    S::OptionalMandatoryNestedBuilders: Default,
    S::OptionalDiscretionaryNestedBuilders: Default,
    Self: CheckAndMoveColumns<S::DefaultColumns>,
{
    type Error = <Self as CheckAndMoveColumns<S::DefaultColumns>>::Error;
    type CheckedBundles = TableBuilderBundle<S>;

    #[inline]
    fn check_defaults(&mut self) -> Result<(), Self::Error> {
        <Self as CheckAndMoveColumns<S::DefaultColumns>>::check_and_move_columns(self)
    }

    #[inline]
    fn into_checked(self) -> Self::CheckedBundles {
        TableBuilderBundle::from_checked_model(self.checked)
    }
}

impl<C, S> SetColumn<C> for DefaultBundle<S>
where
    S: BundlableTableExt,
    C: TypedColumn<Table = S>,
    S::NewValues: SetColumn<C>,
{
    #[inline]
    fn set_column(&mut self, value: impl Into<C::ColumnType>) -> &mut Self {
        <S::NewValues as SetColumn<C>>::set_column(&mut self.raw, value);
        self
    }
}

/// Stages the complete ancestor graph of default values for a table builder.
pub struct DefaultBuilder<T: BuildableTable> {
    /// The staged default bundles for the table and its ancestors.
    bundles: T::NestedDefaultBundles,
}

impl<T: BuildableTable> DefaultBuilder<T> {
    /// Stages the declared defaults of the ancestor graph.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{BuildDefaults, DefaultBuilder};
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let mut staged = DefaultBuilder::<profiles::table>::new();
    /// staged.check_defaults()?;
    /// let profile = staged.finish().display_name("Ada").visits(3).insert(&mut conn)?;
    /// assert_eq!(profile.display_name, "Ada");
    /// assert_eq!(profile.visits, 3);
    /// let user = users::table.find(profile.id).first::<User>(&mut conn)?;
    /// assert_eq!(user.name(), "Ada");
    /// assert_eq!(*user.age(), 18);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { bundles: Default::default() }
    }

    /// Returns a builder containing only successfully checked values.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{BuildDefaults, DefaultBuilder};
    /// use schema::*;
    ///
    /// let mut staged = DefaultBuilder::<users::table>::new();
    /// assert!(staged.check_defaults().is_ok());
    /// let builder = staged.finish();
    /// assert_eq!(builder.may_get_column::<users::age>(), Some(18));
    /// assert_eq!(builder.may_get_column::<users::name>(), None);
    /// # }
    /// ```
    pub fn finish(self) -> TableBuilder<T> {
        TableBuilder::from_bundles(<T::NestedDefaultBundles as BuildDefaults>::into_checked(
            self.bundles,
        ))
    }
}

impl<T: BuildableTable> Default for DefaultBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BuildableTable> BuildDefaults for DefaultBuilder<T> {
    type Error = <T::NestedDefaultBundles as BuildDefaults>::Error;
    type CheckedBundles = TableBuilder<T>;

    #[inline]
    fn check_defaults(&mut self) -> Result<(), Self::Error> {
        <T::NestedDefaultBundles as BuildDefaults>::check_defaults(&mut self.bundles)
    }

    #[inline]
    fn into_checked(self) -> Self::CheckedBundles {
        self.finish()
    }
}

impl<C, T> SetColumn<C> for DefaultBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: VerticalSameAsGroup,
    C::Table: AncestorOfIndex<T> + BundlableTableExt,
    T::NestedDefaultBundles: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = DefaultBundle<C::Table>,
        >,
    DefaultBundle<C::Table>: SetColumn<C>,
    Self: SetHomogeneousNestedColumns<C::ValueType, C::VerticalSameAsNestedColumns>,
{
    #[inline]
    fn set_column(&mut self, value: impl Into<C::ColumnType>) -> &mut Self {
        let value = value.into();
        self.set_homogeneous_nested_columns(&value);
        self.bundles.nested_index_mut().set_column(value);
        self
    }
}

/// Validates and moves a column group into checked state.
pub trait CheckAndMoveColumns<CS: NestedColumns> {
    /// The validation error of the column group.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Validates present non-null values and skips values already moved.
    ///
    /// # Errors
    ///
    /// Returns the first column validation failure.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{CheckAndMoveColumns, DefaultBundle, SetColumn};
    /// use schema::*;
    ///
    /// let mut staged = DefaultBundle::<users::table>::new();
    /// assert_eq!(CheckAndMoveColumns::<(users::age,)>::check_and_move_columns(&mut staged), Ok(()),);
    /// let mut staged = DefaultBundle::<users::table>::new();
    /// SetColumn::<users::age>::set_column(&mut staged, 16);
    /// assert!(matches!(
    ///     CheckAndMoveColumns::<(users::age,)>::check_and_move_columns(&mut staged),
    ///     Err(ValidationError::AgeTooYoung),
    /// ));
    /// # }
    /// ```
    fn check_and_move_columns(&mut self) -> Result<(), Self::Error>;
}

impl<S: BundlableTableExt> CheckAndMoveColumns<()> for DefaultBundle<S> {
    type Error = Infallible;

    #[inline]
    fn check_and_move_columns(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<C, S> CheckAndMoveColumns<(C,)> for DefaultBundle<S>
where
    S: BundlableTableExt,
    C: TypedColumn<Table = S>,
    S::NewValues: MayGetColumn<C> + ValidateColumn<C> + TakeColumn<C> + SetColumn<C>,
{
    type Error = <S::NewValues as ValidateColumn<C>>::Error;

    #[inline]
    fn check_and_move_columns(&mut self) -> Result<(), Self::Error> {
        self.check_and_move_one::<C>()
    }
}

impl<C, Tail, S> CheckAndMoveColumns<(C, Tail)> for DefaultBundle<S>
where
    S: BundlableTableExt,
    C: TypedColumn<Table = S>,
    S::NewValues: MayGetColumn<C> + ValidateColumn<C> + TakeColumn<C> + SetColumn<C>,
    Tail: NestedColumns,
    (C, Tail): NestedColumns,
    Self: CheckAndMoveColumns<(C,)> + CheckAndMoveColumns<Tail>,
{
    type Error = EitherValidationError<
        <Self as CheckAndMoveColumns<(C,)>>::Error,
        <Self as CheckAndMoveColumns<Tail>>::Error,
    >;

    #[inline]
    fn check_and_move_columns(&mut self) -> Result<(), Self::Error> {
        <Self as CheckAndMoveColumns<(C,)>>::check_and_move_columns(self)
            .map_err(EitherValidationError::Left)?;
        <Self as CheckAndMoveColumns<Tail>>::check_and_move_columns(self)
            .map_err(EitherValidationError::Right)
    }
}

impl<C, T> CheckAndMoveColumns<(C,)> for DefaultBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTableExt,
    T::NestedDefaultBundles: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = DefaultBundle<C::Table>,
        >,
    DefaultBundle<C::Table>: CheckAndMoveColumns<(C,)>,
{
    type Error = <DefaultBundle<C::Table> as CheckAndMoveColumns<(C,)>>::Error;

    #[inline]
    fn check_and_move_columns(&mut self) -> Result<(), Self::Error> {
        <DefaultBundle<C::Table> as CheckAndMoveColumns<(C,)>>::check_and_move_columns(
            self.bundles.nested_index_mut(),
        )
    }
}

impl<C, Tail, T> CheckAndMoveColumns<(C, Tail)> for DefaultBuilder<T>
where
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T> + BundlableTableExt,
    T::NestedDefaultBundles: NestedTupleIndexMut<
            <C::Table as AncestorOfIndex<T>>::Idx,
            Element = DefaultBundle<C::Table>,
        >,
    DefaultBundle<C::Table>: CheckAndMoveColumns<(C,)>,
    Tail: NestedColumns,
    (C, Tail): NestedColumns,
    Self: CheckAndMoveColumns<(C,)> + CheckAndMoveColumns<Tail>,
{
    type Error = EitherValidationError<
        <Self as CheckAndMoveColumns<(C,)>>::Error,
        <Self as CheckAndMoveColumns<Tail>>::Error,
    >;

    #[inline]
    fn check_and_move_columns(&mut self) -> Result<(), Self::Error> {
        <Self as CheckAndMoveColumns<(C,)>>::check_and_move_columns(self)
            .map_err(EitherValidationError::Left)?;
        <Self as CheckAndMoveColumns<Tail>>::check_and_move_columns(self)
            .map_err(EitherValidationError::Right)
    }
}

impl BuildDefaults for () {
    type Error = Infallible;
    type CheckedBundles = ();

    #[inline]
    fn check_defaults(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn into_checked(self) -> Self::CheckedBundles {}
}

impl<S> BuildDefaults for (DefaultBundle<S>,)
where
    S: BundlableTableExt,
    DefaultBundle<S>: BuildDefaults<CheckedBundles = TableBuilderBundle<S>>,
{
    type Error = <DefaultBundle<S> as BuildDefaults>::Error;
    type CheckedBundles = (TableBuilderBundle<S>,);

    #[inline]
    fn check_defaults(&mut self) -> Result<(), Self::Error> {
        self.0.check_defaults()
    }

    #[inline]
    fn into_checked(self) -> Self::CheckedBundles {
        (self.0.into_checked(),)
    }
}

impl<H, Tail> BuildDefaults for (DefaultBundle<H>, Tail)
where
    H: BundlableTableExt,
    DefaultBundle<H>: BuildDefaults<CheckedBundles = TableBuilderBundle<H>>,
    Tail: BuildDefaults,
{
    type Error = EitherValidationError<
        <DefaultBundle<H> as BuildDefaults>::Error,
        <Tail as BuildDefaults>::Error,
    >;
    type CheckedBundles = (TableBuilderBundle<H>, <Tail as BuildDefaults>::CheckedBundles);

    #[inline]
    fn check_defaults(&mut self) -> Result<(), Self::Error> {
        self.0.check_defaults().map_err(EitherValidationError::Left)?;
        self.1.check_defaults().map_err(EitherValidationError::Right)
    }

    #[inline]
    fn into_checked(self) -> Self::CheckedBundles {
        (self.0.into_checked(), self.1.into_checked())
    }
}
