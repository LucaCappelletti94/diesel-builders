//! Typed field and record validation protocols.

use std::convert::Infallible;

use tuplities::prelude::IntoNestedTupleOption;

use crate::{
    MayGetColumn, NestedColumns, OptionalRef, SetColumn, TableExt, TypedColumn, ValidateColumn,
    columns::HomogeneouslyTypedNestedColumns,
};

/// A typed validation failure from either of two stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EitherValidationError<L, R> {
    /// The first stage failed.
    Left(L),
    /// The second stage failed.
    Right(R),
}

impl<L: std::fmt::Display, R: std::fmt::Display> std::fmt::Display for EitherValidationError<L, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(error) => error.fmt(f),
            Self::Right(error) => error.fmt(f),
        }
    }
}

impl<L: std::error::Error + 'static, R: std::error::Error + 'static> std::error::Error
    for EitherValidationError<L, R>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Left(error) => error,
            Self::Right(error) => error,
        })
    }
}

impl<L: Into<Infallible>, R: Into<Infallible>> From<EitherValidationError<L, R>> for Infallible {
    fn from(error: EitherValidationError<L, R>) -> Self {
        match error {
            EitherValidationError::Left(error) => error.into(),
            EitherValidationError::Right(error) => error.into(),
        }
    }
}

impl<L: diesel::result::DatabaseErrorInformation, R: diesel::result::DatabaseErrorInformation>
    diesel::result::DatabaseErrorInformation for EitherValidationError<L, R>
{
    fn message(&self) -> &str {
        match self {
            Self::Left(error) => error.message(),
            Self::Right(error) => error.message(),
        }
    }

    fn details(&self) -> Option<&str> {
        match self {
            Self::Left(error) => error.details(),
            Self::Right(error) => error.details(),
        }
    }

    fn hint(&self) -> Option<&str> {
        match self {
            Self::Left(error) => error.hint(),
            Self::Right(error) => error.hint(),
        }
    }

    fn table_name(&self) -> Option<&str> {
        match self {
            Self::Left(error) => error.table_name(),
            Self::Right(error) => error.table_name(),
        }
    }

    fn column_name(&self) -> Option<&str> {
        match self {
            Self::Left(error) => error.column_name(),
            Self::Right(error) => error.column_name(),
        }
    }

    fn constraint_name(&self) -> Option<&str> {
        match self {
            Self::Left(error) => error.constraint_name(),
            Self::Right(error) => error.constraint_name(),
        }
    }

    fn statement_position(&self) -> Option<i32> {
        match self {
            Self::Left(error) => error.statement_position(),
            Self::Right(error) => error.statement_position(),
        }
    }
}

/// Moves a column out of raw values.
pub trait TakeColumn<C: TypedColumn> {
    /// Removes and returns the stored column value.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::TakeColumn;
    /// use schema::*;
    ///
    /// let mut values = user_values("Ada", 20, Some("Ace"));
    /// assert_eq!(TakeColumn::<users::name>::take_column(&mut values), Some("Ada".to_string()),);
    /// assert_eq!(TakeColumn::<users::name>::take_column(&mut values), None);
    /// assert_eq!(
    ///     TakeColumn::<users::nickname>::take_column(&mut values),
    ///     Some(Some("Ace".to_string())),
    /// );
    /// let mut grace = user_values("Grace", 30, None);
    /// assert_eq!(TakeColumn::<users::nickname>::take_column(&mut grace), Some(None),);
    /// # }
    /// ```
    fn take_column(&mut self) -> Option<C::ColumnType>;
}

/// Validates relationships between the final values of a record.
pub trait ValidateRecord<T: TableExt> {
    /// Checks the final record context.
    ///
    /// # Errors
    /// Returns the record's declared validation error.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::SetColumn;
    /// use schema::*;
    ///
    /// let mut record = intervals::table::empty_new_values();
    /// SetColumn::<intervals::start>::set_column(&mut record, 1);
    /// SetColumn::<intervals::end>::set_column(&mut record, 5);
    /// assert_eq!(record.validate_record(), Ok(()));
    ///
    /// let mut record = intervals::table::empty_new_values();
    /// SetColumn::<intervals::start>::set_column(&mut record, 5);
    /// SetColumn::<intervals::end>::set_column(&mut record, 5);
    /// assert_eq!(record.validate_record(), Err(RecordError));
    /// # }
    /// ```
    fn validate_record(&self) -> Result<(), T::RecordError>;
}

/// Validates one nullable value with its column validator.
fn validate_value<T, C>(value: &C::ColumnType) -> Result<(), <T as ValidateColumn<C>>::Error>
where
    T: ValidateColumn<C> + ?Sized,
    C: TypedColumn,
{
    if let Some(value) = value.as_optional_ref() {
        T::validate_column(value)?;
    }
    Ok(())
}

/// Checks present stored values with their intrinsic validators.
pub trait ValidateStoredValues<CS: NestedColumns> {
    /// The possible errors of these columns.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Validates each present non-null value.
    ///
    /// # Errors
    /// Returns the first column validation failure.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{SetColumn, ValidateStoredValues};
    /// use schema::*;
    ///
    /// type NewValues = <invalid_defaults::table as TableExt>::NewValues;
    /// type NewRecord = <invalid_defaults::table as TableExt>::NewRecord;
    ///
    /// let empty = invalid_defaults::table::empty_new_values();
    /// assert_eq!(
    ///     <NewValues as ValidateStoredValues<NewRecord>>::validate_stored_values(&empty),
    ///     Ok(()),
    /// );
    ///
    /// let mut valid = invalid_defaults::table::empty_new_values();
    /// SetColumn::<invalid_defaults::value>::set_column(&mut valid, 5);
    /// assert_eq!(
    ///     <NewValues as ValidateStoredValues<NewRecord>>::validate_stored_values(&valid),
    ///     Ok(()),
    /// );
    ///
    /// let mut invalid = invalid_defaults::table::empty_new_values();
    /// SetColumn::<invalid_defaults::value>::set_column(&mut invalid, -5);
    /// assert_eq!(
    ///     <NewValues as ValidateStoredValues<NewRecord>>::validate_stored_values(&invalid),
    ///     Err(ValidationError::NegativeValue),
    /// );
    /// # }
    /// ```
    fn validate_stored_values(&self) -> Result<(), Self::Error>;
}

impl<T> ValidateStoredValues<()> for T {
    type Error = Infallible;
    fn validate_stored_values(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T, C> ValidateStoredValues<(C,)> for T
where
    C: TypedColumn<Table: TableExt>,
    T: MayGetColumn<C> + ValidateColumn<C>,
{
    type Error = <T as ValidateColumn<C>>::Error;
    fn validate_stored_values(&self) -> Result<(), Self::Error> {
        if let Some(value) = <Self as MayGetColumn<C>>::may_get_column_ref(self) {
            validate_value::<Self, C>(value)?;
        }
        Ok(())
    }
}

impl<T, C, Tail> ValidateStoredValues<(C, Tail)> for T
where
    C: TypedColumn,
    Tail: NestedColumns,
    (C, Tail): NestedColumns,
    T: MayGetColumn<C> + ValidateColumn<C> + ValidateStoredValues<Tail>,
{
    type Error = EitherValidationError<
        <T as ValidateColumn<C>>::Error,
        <T as ValidateStoredValues<Tail>>::Error,
    >;
    fn validate_stored_values(&self) -> Result<(), Self::Error> {
        if let Some(value) = <Self as MayGetColumn<C>>::may_get_column_ref(self) {
            validate_value::<Self, C>(value).map_err(EitherValidationError::Left)?;
        }
        <Self as ValidateStoredValues<Tail>>::validate_stored_values(self)
            .map_err(EitherValidationError::Right)
    }
}

/// Checks candidate values before any column assignment.
pub trait ValidateIncomingValues<CS: NestedColumns> {
    /// The possible errors of these columns.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Validates each non-null candidate.
    ///
    /// # Errors
    /// Returns the first column validation failure.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::ValidateIncomingValues;
    /// use schema::*;
    ///
    /// type NewValues = <users::table as TableExt>::NewValues;
    ///
    /// assert_eq!(
    ///     <NewValues as ValidateIncomingValues<(users::age,)>>::validate_incoming_values(&(20,),),
    ///     Ok(()),
    /// );
    /// assert_eq!(
    ///     <NewValues as ValidateIncomingValues<(users::age,)>>::validate_incoming_values(&(17,),),
    ///     Err(ValidationError::AgeTooYoung),
    /// );
    /// # }
    /// ```
    fn validate_incoming_values(values: &CS::NestedTupleColumnType) -> Result<(), Self::Error>;
}

impl<T> ValidateIncomingValues<()> for T {
    type Error = Infallible;
    fn validate_incoming_values(_values: &()) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T, C> ValidateIncomingValues<(C,)> for T
where
    C: TypedColumn<Table: TableExt>,
    T: ValidateColumn<C>,
{
    type Error = <T as ValidateColumn<C>>::Error;
    fn validate_incoming_values((value,): &(C::ColumnType,)) -> Result<(), Self::Error> {
        validate_value::<Self, C>(value)
    }
}

impl<T, C, Tail> ValidateIncomingValues<(C, Tail)> for T
where
    C: TypedColumn,
    Tail: NestedColumns,
    (C, Tail): NestedColumns<NestedTupleColumnType = (C::ColumnType, Tail::NestedTupleColumnType)>,
    T: ValidateColumn<C> + ValidateIncomingValues<Tail>,
{
    type Error = EitherValidationError<
        <T as ValidateColumn<C>>::Error,
        <T as ValidateIncomingValues<Tail>>::Error,
    >;
    fn validate_incoming_values(
        (head, tail): &(C::ColumnType, Tail::NestedTupleColumnType),
    ) -> Result<(), Self::Error> {
        validate_value::<Self, C>(head).map_err(EitherValidationError::Left)?;
        <Self as ValidateIncomingValues<Tail>>::validate_incoming_values(tail)
            .map_err(EitherValidationError::Right)
    }
}

/// Copies validated values into their columns.
trait AssignValues<CS: NestedColumns> {
    /// Copies the values into their columns.
    fn assign_values(&mut self, values: CS::NestedTupleColumnType);
}

impl<T> AssignValues<()> for T {
    fn assign_values(&mut self, _values: ()) {}
}

impl<T, C> AssignValues<(C,)> for T
where
    C: TypedColumn<Table: TableExt>,
    T: SetColumn<C>,
{
    fn assign_values(&mut self, (value,): (C::ColumnType,)) {
        <Self as SetColumn<C>>::set_column(self, value);
    }
}

impl<T, C, Tail> AssignValues<(C, Tail)> for T
where
    C: TypedColumn,
    Tail: NestedColumns,
    (C, Tail): NestedColumns<NestedTupleColumnType = (C::ColumnType, Tail::NestedTupleColumnType)>,
    T: SetColumn<C> + AssignValues<Tail>,
{
    fn assign_values(&mut self, (head, tail): (C::ColumnType, Tail::NestedTupleColumnType)) {
        <Self as SetColumn<C>>::set_column(self, head);
        <Self as AssignValues<Tail>>::assign_values(self, tail);
    }
}

/// Validates a candidate group before storing any of its values.
pub trait TrySetValues<CS: NestedColumns> {
    /// The possible errors of these columns.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Checks and assigns the candidate group.
    ///
    /// # Errors
    /// Returns the unchanged candidate group and its validation error.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, TrySetValues};
    /// use schema::*;
    ///
    /// type NewValues = <users::table as TableExt>::NewValues;
    /// let mut values = user_values("Ada", 20, None);
    /// let rejected = <NewValues as TrySetValues<(users::age,)>>::try_set_values(&mut values, (17,));
    /// assert!(matches!(rejected, Err(((17,), ValidationError::AgeTooYoung))));
    /// assert!(matches!(
    ///     <NewValues as TrySetValues<(users::age,)>>::try_set_values(&mut values, (25,)),
    ///     Ok(_),
    /// ));
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column(&values), Some(25));
    /// # }
    /// ```
    fn try_set_values(
        &mut self,
        values: CS::NestedTupleColumnType,
    ) -> Result<&mut Self, (CS::NestedTupleColumnType, Self::Error)>;
}

impl<T, CS> TrySetValues<CS> for T
where
    CS: NestedColumns,
    T: ValidateIncomingValues<CS> + AssignValues<CS>,
{
    type Error = <T as ValidateIncomingValues<CS>>::Error;
    fn try_set_values(
        &mut self,
        values: CS::NestedTupleColumnType,
    ) -> Result<&mut Self, (CS::NestedTupleColumnType, Self::Error)> {
        if let Err(error) = <Self as ValidateIncomingValues<CS>>::validate_incoming_values(&values)
        {
            return Err((values, error));
        }
        <Self as AssignValues<CS>>::assign_values(self, values);
        Ok(self)
    }
}

/// Checks optional candidate values before any column assignment.
pub trait ValidateOptionalIncomingValues<CS: NestedColumns> {
    /// The possible errors of these columns.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Validates each present non-null candidate.
    ///
    /// # Errors
    /// Returns the first column validation failure.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::ValidateOptionalIncomingValues;
    /// use schema::*;
    ///
    /// type NewValues = <users::table as TableExt>::NewValues;
    ///
    /// assert_eq!(
    ///     <NewValues as ValidateOptionalIncomingValues<(users::nickname,)>>::
    ///         validate_optional_incoming_values(&(None,)),
    ///     Ok(()),
    /// );
    /// assert_eq!(
    ///     <NewValues as ValidateOptionalIncomingValues<(users::nickname,)>>::
    ///         validate_optional_incoming_values(&(Some(Some("Ace".to_string())),)),
    ///     Ok(()),
    /// );
    /// assert_eq!(
    ///     <NewValues as ValidateOptionalIncomingValues<(users::nickname,)>>::
    ///         validate_optional_incoming_values(&(Some(None),)),
    ///     Ok(()),
    /// );
    /// assert_eq!(
    ///     <NewValues as ValidateOptionalIncomingValues<(users::nickname,)>>::
    ///         validate_optional_incoming_values(&(Some(Some(String::new())),)),
    ///     Err(ValidationError::EmptyNickname),
    /// );
    /// # }
    /// ```
    fn validate_optional_incoming_values(
        values: &<CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<(), Self::Error>;
}

impl<T> ValidateOptionalIncomingValues<()> for T {
    type Error = Infallible;
    fn validate_optional_incoming_values(_values: &()) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<T, C> ValidateOptionalIncomingValues<(C,)> for T
where
    C: TypedColumn<Table: TableExt>,
    T: ValidateColumn<C>,
{
    type Error = <T as ValidateColumn<C>>::Error;
    fn validate_optional_incoming_values(
        (value,): &(Option<C::ColumnType>,),
    ) -> Result<(), Self::Error> {
        if let Some(value) = value {
            validate_value::<Self, C>(value)?;
        }
        Ok(())
    }
}

impl<T, C, Tail> ValidateOptionalIncomingValues<(C, Tail)> for T
where
    C: TypedColumn,
    Tail: NestedColumns,
    (C, Tail): NestedColumns<NestedTupleColumnType = (C::ColumnType, Tail::NestedTupleColumnType)>,
    T: ValidateColumn<C> + ValidateOptionalIncomingValues<Tail>,
{
    type Error = EitherValidationError<
        <T as ValidateColumn<C>>::Error,
        <T as ValidateOptionalIncomingValues<Tail>>::Error,
    >;
    fn validate_optional_incoming_values(
        (head, tail): &(
            Option<C::ColumnType>,
            <Tail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) -> Result<(), Self::Error> {
        if let Some(head) = head {
            validate_value::<Self, C>(head).map_err(EitherValidationError::Left)?;
        }
        <Self as ValidateOptionalIncomingValues<Tail>>::validate_optional_incoming_values(tail)
            .map_err(EitherValidationError::Right)
    }
}

/// Copies validated values into their optional columns.
trait AssignOptionalValues<CS: NestedColumns> {
    /// Copies the values into their optional columns.
    fn assign_optional_values(
        &mut self,
        values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    );
}

impl<T> AssignOptionalValues<()> for T {
    fn assign_optional_values(&mut self, _values: ()) {}
}

impl<T, C> AssignOptionalValues<(C,)> for T
where
    C: TypedColumn<Table: TableExt>,
    T: SetColumn<C>,
{
    fn assign_optional_values(&mut self, (value,): (Option<C::ColumnType>,)) {
        if let Some(value) = value {
            <Self as SetColumn<C>>::set_column(self, value);
        }
    }
}

impl<T, C, Tail> AssignOptionalValues<(C, Tail)> for T
where
    C: TypedColumn,
    Tail: NestedColumns,
    (C, Tail): NestedColumns<NestedTupleColumnType = (C::ColumnType, Tail::NestedTupleColumnType)>,
    T: SetColumn<C> + AssignOptionalValues<Tail>,
{
    fn assign_optional_values(
        &mut self,
        (head, tail): (
            Option<C::ColumnType>,
            <Tail::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
        ),
    ) {
        if let Some(head) = head {
            <Self as SetColumn<C>>::set_column(self, head);
        }
        <Self as AssignOptionalValues<Tail>>::assign_optional_values(self, tail);
    }
}

/// Validates optional candidates before storing any present value.
pub trait TryMaySetValues<CS: NestedColumns> {
    /// The possible errors of these columns.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Checks and assigns the optional candidate group.
    ///
    /// # Errors
    /// Returns the unchanged candidate group and its validation error.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, TryMaySetValues};
    /// use schema::*;
    ///
    /// type NewValues = <users::table as TableExt>::NewValues;
    /// let mut values = user_values("Grace", 30, None);
    ///
    /// let rejected = <NewValues as TryMaySetValues<(users::nickname,)>>::try_may_set_values(
    ///     &mut values,
    ///     (Some(Some(String::new())),),
    /// );
    /// assert!(matches!(rejected, Err(((Some(Some(_)),), ValidationError::EmptyNickname)),));
    /// assert_eq!(MayGetColumn::<users::nickname>::may_get_column(&values), Some(None),);
    ///
    /// assert!(matches!(
    ///     <NewValues as TryMaySetValues<(users::nickname,)>>::try_may_set_values(
    ///         &mut values,
    ///         (Some(Some("Ace".to_string())),),
    ///     ),
    ///     Ok(_),
    /// ));
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string())),
    /// );
    ///
    /// assert!(matches!(
    ///     <NewValues as TryMaySetValues<(users::nickname,)>>::try_may_set_values(
    ///         &mut values,
    ///         (None,),
    ///     ),
    ///     Ok(_),
    /// ));
    /// assert_eq!(
    ///     MayGetColumn::<users::nickname>::may_get_column(&values),
    ///     Some(Some("Ace".to_string())),
    /// );
    ///
    /// assert!(matches!(
    ///     <NewValues as TryMaySetValues<(users::nickname,)>>::try_may_set_values(
    ///         &mut values,
    ///         (Some(None),),
    ///     ),
    ///     Ok(_),
    /// ));
    /// assert_eq!(MayGetColumn::<users::nickname>::may_get_column(&values), Some(None),);
    /// # }
    /// ```
    fn try_may_set_values(
        &mut self,
        values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<
        &mut Self,
        (<CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions, Self::Error),
    >;
}

impl<T, CS> TryMaySetValues<CS> for T
where
    CS: NestedColumns,
    T: ValidateOptionalIncomingValues<CS> + AssignOptionalValues<CS>,
{
    type Error = <T as ValidateOptionalIncomingValues<CS>>::Error;
    fn try_may_set_values(
        &mut self,
        values: <CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
    ) -> Result<
        &mut Self,
        (<CS::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions, Self::Error),
    > {
        if let Err(error) =
            <Self as ValidateOptionalIncomingValues<CS>>::validate_optional_incoming_values(&values)
        {
            return Err((values, error));
        }
        <Self as AssignOptionalValues<CS>>::assign_optional_values(self, values);
        Ok(self)
    }
}

/// Expands one value across a homogeneous column group.
trait ConvertHomogeneousValues<Type, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// Expands the value into the group's column tuple.
    fn convert_homogeneous_values(value: &Type) -> CS::NestedTupleColumnType;
}

impl<T, Type> ConvertHomogeneousValues<Type, ()> for T {
    fn convert_homogeneous_values(_value: &Type) {}
}

impl<T, Type, C> ConvertHomogeneousValues<Type, (C,)> for T
where
    Type: Clone,
    C: TypedColumn<ColumnType: From<Type>, Table: TableExt>,
{
    fn convert_homogeneous_values(value: &Type) -> (C::ColumnType,) {
        (value.clone().into(),)
    }
}

impl<T, Type, C, Tail> ConvertHomogeneousValues<Type, (C, Tail)> for T
where
    Type: Clone,
    C: TypedColumn<ColumnType: From<Type>>,
    Tail: HomogeneouslyTypedNestedColumns<Type>,
    (C, Tail): HomogeneouslyTypedNestedColumns<Type>
        + NestedColumns<NestedTupleColumnType = (C::ColumnType, Tail::NestedTupleColumnType)>,
    T: ConvertHomogeneousValues<Type, Tail>,
{
    fn convert_homogeneous_values(value: &Type) -> (C::ColumnType, Tail::NestedTupleColumnType) {
        (
            value.clone().into(),
            <Self as ConvertHomogeneousValues<Type, Tail>>::convert_homogeneous_values(value),
        )
    }
}

/// Validates converted propagation targets before storing any of them.
pub trait TrySetHomogeneousValues<Type, CS: HomogeneouslyTypedNestedColumns<Type>> {
    /// The possible errors of these columns.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Checks and propagates a present shared value.
    ///
    /// # Errors
    /// Returns the first target validation failure.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, TrySetHomogeneousValues};
    /// use schema::*;
    ///
    /// type NewValues = <intervals::table as TableExt>::NewValues;
    /// type Columns = (intervals::start, (intervals::end,));
    ///
    /// let mut values = intervals::table::empty_new_values();
    /// assert!(matches!(
    ///     <NewValues as TrySetHomogeneousValues<i32, Columns>>::try_set_homogeneous_values(
    ///         &mut values,
    ///         &None::<i32>,
    ///     ),
    ///     Ok(_),
    /// ));
    /// assert_eq!(MayGetColumn::<intervals::start>::may_get_column(&values), None);
    ///
    /// let mut values = intervals::table::empty_new_values();
    /// assert!(matches!(
    ///     <NewValues as TrySetHomogeneousValues<i32, Columns>>::try_set_homogeneous_values(
    ///         &mut values,
    ///         &5,
    ///     ),
    ///     Ok(_),
    /// ));
    /// assert_eq!(MayGetColumn::<intervals::start>::may_get_column(&values), Some(5));
    /// assert_eq!(MayGetColumn::<intervals::end>::may_get_column(&values), Some(5));
    /// # }
    /// ```
    fn try_set_homogeneous_values(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Self::Error>;
}

impl<T, Type, CS> TrySetHomogeneousValues<Type, CS> for T
where
    CS: HomogeneouslyTypedNestedColumns<Type>,
    T: ConvertHomogeneousValues<Type, CS> + TrySetValues<CS>,
{
    type Error = <T as TrySetValues<CS>>::Error;
    fn try_set_homogeneous_values(
        &mut self,
        value: &impl OptionalRef<Type>,
    ) -> Result<&mut Self, Self::Error> {
        if let Some(value) = value.as_optional_ref() {
            let converted =
                <Self as ConvertHomogeneousValues<Type, CS>>::convert_homogeneous_values(value);
            <Self as TrySetValues<CS>>::try_set_values(self, converted)
                .map_err(|(_, error)| error)?;
        }
        Ok(self)
    }
}
