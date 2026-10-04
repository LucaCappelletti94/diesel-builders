//! Submodule for the completed table builder bundle and related impls.

use diesel::{Column, Insertable, RunQueryDsl, associations::HasTable};
use tuplities::prelude::*;

use crate::{
    BuildableTable, BuilderError, BuilderResult, DiscretionarySameAsIndex, EitherValidationError,
    HasNestedTables, HasTableExt, IncompleteBuilderError, MandatorySameAsIndex, NestedColumns,
    NestedTables, TableBuilder, TableBuilderBundle, TableExt, TryMaySetValues, TrySetColumn,
    TrySetDiscretionarySameAsColumn, TrySetMandatorySameAsColumn, TrySetValues,
    TupleGetNestedColumns, TupleMayGetNestedColumns, TypedColumn, TypedNestedTuple, ValidateColumn,
    ValidateRecord,
    builder_bundle::{BundlableTableExt, impl_bundle_shared},
    columns::TupleEqAll,
    horizontal_same_as_group::HorizontalSameAsGroupExt,
    insert_transaction::{CheckNestedStoredOptions, InsertBuilder, InsertBundle},
};

#[derive(Debug)]
/// The build-ready variant of a table builder bundle.
pub struct CompletedTableBuilderBundle<T: BundlableTableExt> {
    /// The insertable model for the table.
    insertable_model: T::NewValues,
    /// The mandatory associated builders relative to triangular same-as.
    nested_mandatory_associated_builders: T::MandatoryNestedBuilders,
    /// The discretionary associated builders relative to triangular same-as.
    nested_discretionary_associated_builders: T::OptionalDiscretionaryNestedBuilders,
}

impl_bundle_shared!(CompletedTableBuilderBundle);

impl<Key, C> crate::mutation_same_as::PrepareMandatoryColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    Key: MandatorySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>,
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::MandatoryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = TableBuilder<C::Table>>,
    TableBuilder<C::Table>: crate::mutation::PrepareColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;
    type Prepared = <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::Prepared;

    fn has_mandatory_target(&self, context: &crate::mutation::MutationContext) -> bool {
        !context.excludes_mandatory(
            std::ptr::from_ref(self).cast::<()>(),
            <Key::Idx as typenum::Unsigned>::USIZE,
        )
    }

    fn prepare_mandatory_column(
        &self,
        value: C::ColumnType,
        context: &crate::mutation::MutationContext,
    ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)> {
        <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::prepare_column(
            self.nested_mandatory_associated_builders.nested_index(),
            value,
            context,
        )
    }

    fn apply_mandatory_column(&mut self, prepared: Self::Prepared) {
        <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::apply_column(
            self.nested_mandatory_associated_builders.nested_index_mut(),
            prepared,
        );
    }
}

impl<Key: MandatorySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>, C>
    TrySetMandatorySameAsColumn<Key, C> for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::MandatoryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = TableBuilder<C::Table>>,
    TableBuilder<C::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        self.nested_mandatory_associated_builders.nested_index_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<T> TryFrom<TableBuilderBundle<T>> for CompletedTableBuilderBundle<T>
where
    T: BundlableTableExt,
{
    type Error = IncompleteBuilderError;

    fn try_from(
        value: TableBuilderBundle<T>,
    ) -> Result<CompletedTableBuilderBundle<T>, Self::Error> {
        Ok(CompletedTableBuilderBundle {
            insertable_model: value.insertable_model,
            nested_mandatory_associated_builders: value
                .nested_mandatory_associated_builders
                .transpose_or(T::NestedMandatoryTriangularColumns::NESTED_COLUMN_NAMES)
                .map_err(|column_name| {
                    IncompleteBuilderError::MissingMandatoryTriangularField {
                        table_name: T::TABLE_NAME,
                        field_name: column_name,
                    }
                })?,
            nested_discretionary_associated_builders: value
                .nested_discretionary_associated_builders,
        })
    }
}

/// Trait defining the insertion of a builder into the database.
pub trait RecursiveBundleInsert<Error, Conn>: HasTableExt {
    /// Insert the builder's data into the database using the provided
    /// connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails or if any database constraints
    /// are violated.
    ///
    /// # Examples
    ///
    /// Complete a checked bundle and insert the record it stages.
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::{CompletedTableBuilderBundle, RecursiveBundleInsert, TableBuilderBundle};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let bundle = TableBuilderBundle::<users::table>::try_from_values(user_values("Ada", 20, Some("Ace"))).expect("valid values must be accepted");
    /// let completed: CompletedTableBuilderBundle<users::table> = bundle.try_into()?;
    /// let user = completed.recursive_bundle_insert(&mut conn)?;
    /// assert_eq!(user.get_column::<users::name>(), "Ada");
    /// # Ok(())
    /// # }
    /// ```
    fn recursive_bundle_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableExt>::Model, Error>;
}

impl<T, Conn, MandatoryError, DiscretionaryError> InsertBundle<Conn> for CompletedTableBuilderBundle<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BundlableTableExt,
    T::NewValues: TrySetValues<T::NestedMandatoryTriangularColumns>
        + TryMaySetValues<T::NestedDiscretionaryTriangularColumns>
        + ValidateRecord<T>
        + NestedTupleRef,
    for<'a> <T::NewValues as NestedTupleRef>::Ref<'a>: CheckNestedStoredOptions<
            &'static str,
            SameDepth = <T::NewRecord as NestedColumns>::NestedNames,
        >,
    T::MandatoryNestedBuilders: InsertTuple<Conn, ValidationError = MandatoryError>,
    T::OptionalDiscretionaryNestedBuilders:
        InsertOptionTuple<Conn, ValidationError = DiscretionaryError>,
    T::NewRecord: TupleEqAll<EqAll: FlattenNestedTuple<Flattened: Insertable<T>>> + TypedNestedTuple<NestedTupleColumnType=T::CompletedNewValues>,
    diesel::query_builder::InsertStatement<
        Self::Table,
        <<<T::NewRecord as TupleEqAll>::EqAll as FlattenNestedTuple>::Flattened as Insertable<T>>::Values,
    >: for<'query> diesel::query_dsl::LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    type ValidationError = EitherValidationError<
        MandatoryError,
        EitherValidationError<
            <T::NewValues as TrySetValues<T::NestedMandatoryTriangularColumns>>::Error,
            EitherValidationError<
                DiscretionaryError,
                EitherValidationError<
                    <T::NewValues as TryMaySetValues<T::NestedDiscretionaryTriangularColumns>>::Error,
                    <T as TableExt>::RecordError,
                >,
            >,
        >,
    >;

    fn insert_bundle(
        mut self,
        conn: &mut Conn,
    ) -> BuilderResult<<T as TableExt>::Model, Self::ValidationError> {
        let mandatory_models: T::NestedMandatoryModels = self
            .nested_mandatory_associated_builders
            .insert_tuple(conn)
            .map_err(|error| error.map_validation(EitherValidationError::Left))?;
        let mandatory_primary_keys: T::NestedMandatoryPrimaryKeyTypes =
            mandatory_models.tuple_get_nested_columns();
        self.insertable_model
            .try_set_values(mandatory_primary_keys)
            .map_err(|(_, error)| {
                BuilderError::Validation(EitherValidationError::Right(EitherValidationError::Left(
                    error,
                )))
            })?;
        let discretionary_models: T::OptionalNestedDiscretionaryModels = self
            .nested_discretionary_associated_builders
            .insert_option_tuple(conn)
            .map_err(|error| {
                error.map_validation(|error| {
                    EitherValidationError::Right(EitherValidationError::Right(EitherValidationError::Left(error)))
                })
            })?;
        let discretionary_primary_keys: T::OptionalNestedDiscretionaryPrimaryKeyTypes =
            discretionary_models.tuple_may_get_nested_columns();
        self.insertable_model
            .try_may_set_values(discretionary_primary_keys)
            .map_err(|(_, error)| {
                BuilderError::Validation(EitherValidationError::Right(EitherValidationError::Right(
                    EitherValidationError::Right(EitherValidationError::Left(error)),
                )))
            })?;

        // Check the required fields without moving the stored values, so that
        // the record validator observes the complete final values.
        if let Some(column_name) = self
            .insertable_model
            .nested_tuple_ref()
            .first_missing_with(T::NewRecord::NESTED_COLUMN_NAMES)
        {
            return Err(BuilderError::Incomplete(IncompleteBuilderError::MissingMandatoryField {
                table_name: T::TABLE_NAME,
                field_name: column_name,
            }));
        }

        self.insertable_model
            .validate_record()
            .map_err(|error| {
                BuilderError::Validation(EitherValidationError::Right(EitherValidationError::Right(
                    EitherValidationError::Right(EitherValidationError::Right(error)),
                )))
            })?;

        let columns = T::NewRecord::default();
        let values: T::CompletedNewValues = self
            .insertable_model
            .transpose_or(T::NewRecord::NESTED_COLUMN_NAMES)
            .map_err(|column_name| {
                BuilderError::Incomplete(IncompleteBuilderError::MissingMandatoryField {
                    table_name: T::TABLE_NAME,
                    field_name: column_name,
                })
            })?;

        Ok(diesel::insert_into(T::default())
            .values(columns.eq_all(values).flatten())
            .get_result(conn)?)
    }
}

/// Trait defining the insertion of a tuple of builders into the database.
trait InsertTuple<Conn>: HasNestedTables {
    /// The validation errors of the tuple's builders.
    type ValidationError;

    /// # Errors
    ///
    /// Returns an error if any insertion fails, if any remaining validation
    /// fails, or if any database constraints are violated.
    fn insert_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::NestedModels, Self::ValidationError>;
}

impl<Conn> InsertTuple<Conn> for ()
where
    Conn: diesel::connection::LoadConnection,
{
    type ValidationError = std::convert::Infallible;

    #[inline]
    fn insert_tuple(self, _conn: &mut Conn) -> BuilderResult<(), Self::ValidationError> {
        Ok(())
    }
}

impl<Conn, T> InsertTuple<Conn> for (T,)
where
    Conn: diesel::connection::LoadConnection,
    T: InsertBuilder<Conn> + HasTableExt,
{
    type ValidationError = <T as InsertBuilder<Conn>>::ValidationError;

    #[inline]
    fn insert_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::NestedModels, Self::ValidationError>
    {
        Ok((self.0.insert_builder(conn)?,))
    }
}

impl<Conn, Head, Tail> InsertTuple<Conn> for (Head, Tail)
where
    Conn: diesel::connection::LoadConnection,
    Head: InsertBuilder<Conn> + HasTableExt,
    Tail: InsertTuple<Conn>,
    (Head, Tail): HasNestedTables,
    Self::NestedTables: NestedTables<
        NestedModels = (
            <<Head as HasTable>::Table as TableExt>::Model,
            <Tail::NestedTables as NestedTables>::NestedModels,
        ),
    >,
{
    type ValidationError = EitherValidationError<
        <Head as InsertBuilder<Conn>>::ValidationError,
        <Tail as InsertTuple<Conn>>::ValidationError,
    >;

    #[inline]
    fn insert_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::NestedModels, Self::ValidationError>
    {
        let head = self
            .0
            .insert_builder(conn)
            .map_err(|error| error.map_validation(EitherValidationError::Left))?;
        let tail = self
            .1
            .insert_tuple(conn)
            .map_err(|error| error.map_validation(EitherValidationError::Right))?;
        Ok((head, tail))
    }
}

/// Trait defining the insertion of a tuple of optional builders into the
/// database.
trait InsertOptionTuple<Conn>: HasNestedTables {
    /// The validation errors of the tuple's optional builders.
    type ValidationError;

    /// Insert the tuple of optional builders' data into the database using
    /// the provided connection. If a builder is `None`, the corresponding
    /// model will also be `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if any insertion fails, if any remaining validation
    /// fails, or if any database constraints are violated.
    fn insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<
        <Self::NestedTables as NestedTables>::OptionalNestedModels,
        Self::ValidationError,
    >;
}

impl<Conn> InsertOptionTuple<Conn> for () {
    type ValidationError = std::convert::Infallible;

    #[inline]
    fn insert_option_tuple(
        self,
        _conn: &mut Conn,
    ) -> BuilderResult<
        <Self::NestedTables as NestedTables>::OptionalNestedModels,
        Self::ValidationError,
    > {
        Ok(())
    }
}

impl<Conn, T> InsertOptionTuple<Conn> for (Option<T>,)
where
    T: InsertBuilder<Conn> + HasTable,
{
    type ValidationError = <T as InsertBuilder<Conn>>::ValidationError;

    #[inline]
    fn insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<
        <Self::NestedTables as NestedTables>::OptionalNestedModels,
        Self::ValidationError,
    > {
        Ok((match self.0 {
            Some(builder) => Some(builder.insert_builder(conn)?),
            None => None,
        },))
    }
}

impl<Conn, Head, Tail> InsertOptionTuple<Conn> for (Option<Head>, Tail)
where
    Head: InsertBuilder<Conn>,
    Tail: InsertOptionTuple<Conn>,
    (Option<Head>, Tail): HasNestedTables,
    Self::NestedTables: NestedTables<
        OptionalNestedModels = (
            Option<<Head::Table as TableExt>::Model>,
            <Tail::NestedTables as NestedTables>::OptionalNestedModels,
        ),
    >,
{
    type ValidationError = EitherValidationError<
        <Head as InsertBuilder<Conn>>::ValidationError,
        <Tail as InsertOptionTuple<Conn>>::ValidationError,
    >;

    #[inline]
    fn insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<
        <Self::NestedTables as NestedTables>::OptionalNestedModels,
        Self::ValidationError,
    > {
        let head = match self.0 {
            Some(builder) => {
                Some(
                    builder
                        .insert_builder(conn)
                        .map_err(|error| error.map_validation(EitherValidationError::Left))?,
                )
            }
            None => None,
        };
        let tail = self
            .1
            .insert_option_tuple(conn)
            .map_err(|error| error.map_validation(EitherValidationError::Right))?;
        Ok((head, tail))
    }
}
