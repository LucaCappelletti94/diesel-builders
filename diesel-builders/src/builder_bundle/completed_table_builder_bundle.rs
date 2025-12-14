//! Submodule for the completed table builder bundle and related impls.

use diesel::{Column, Insertable, RunQueryDsl, associations::HasTable};
use tuplities::prelude::*;

use crate::columns::TupleEqAll;
use crate::{
    BuildableTable, BuilderError, BuilderResult, DiscretionarySameAsIndex, HasNestedTables,
    HasTableExt, IncompleteBuilderError, MandatorySameAsIndex, NestedColumns, NestedTables,
    RecursiveBuilderInsert, TableBuilder, TableBuilderBundle, TableExt,
    TryMaySetDiscretionarySameAsColumn, TryMaySetDiscretionarySameAsNestedColumns,
    TryMaySetNestedColumns, TrySetColumn, TrySetMandatorySameAsColumn,
    TrySetMandatorySameAsNestedColumns, TrySetNestedColumns, TupleGetNestedColumns,
    TupleMayGetNestedColumns, TypedColumn, builder_bundle::BundlableTableExt,
    horizontal_same_as_group::HorizontalSameAsGroupExt,
};
use crate::{TypedNestedTuple, ValidateColumn};

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

impl<T> HasTable for CompletedTableBuilderBundle<T>
where
    T: BundlableTableExt,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, C> ValidateColumn<C> for CompletedTableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: HorizontalSameAsGroupExt,
    T::NewValues: ValidateColumn<C>,
{
    type Error = <T::NewValues as ValidateColumn<C>>::Error;

    #[inline]
    fn validate_column_in_context(&self, value: &C::ColumnType) -> Result<(), Self::Error> {
        self.insertable_model.validate_column_in_context(value)
    }
}

impl<T, C> TrySetColumn<C> for CompletedTableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: HorizontalSameAsGroupExt,
    Self: TryMaySetDiscretionarySameAsNestedColumns<
            C::ColumnType,
            <T::NewValues as ValidateColumn<C>>::Error,
            C::NestedDiscretionaryHorizontalKeys,
            C::NestedDiscretionaryForeignColumns,
        > + TrySetMandatorySameAsNestedColumns<
            C::ColumnType,
            <T::NewValues as ValidateColumn<C>>::Error,
            C::NestedMandatoryHorizontalKeys,
            C::NestedMandatoryForeignColumns,
        >,
    T::NewValues: TrySetColumn<C>,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType> + Clone,
    ) -> Result<&mut Self, Self::Error> {
        let value = value.into();
        self.validate_column_in_context(&value)?;
        self.try_may_set_discretionary_same_as_nested_columns(&value)?;
        self.try_set_mandatory_same_as_columns(&value)?;
        self.insertable_model.try_set_column(value)?;
        Ok(self)
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
        value: impl Into<C::ColumnType> + Clone,
    ) -> Result<&mut Self, Self::Error> {
        self.nested_mandatory_associated_builders
            .nested_index_mut()
            .try_set_column(value)?;
        Ok(self)
    }
}

impl<Key: DiscretionarySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>, C>
    TryMaySetDiscretionarySameAsColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::OptionalDiscretionaryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
    TableBuilder<C::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType> + Clone,
    ) -> Result<&mut Self, Self::Error> {
        if let Some(builder) = self
            .nested_discretionary_associated_builders
            .nested_index_mut()
            .as_mut()
        {
            builder.try_set_column(value)?;
        }
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
                    IncompleteBuilderError::MissingMandatoryTriangularField(column_name)
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
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails or if any database constraints
    /// are violated.
    fn recursive_bundle_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self as HasTable>::Table as TableExt>::Model, Error>;
}

impl<T, Error, Conn> RecursiveBundleInsert<Error, Conn> for CompletedTableBuilderBundle<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BundlableTableExt,
    T::NewValues: TrySetNestedColumns<Error, T::NestedMandatoryTriangularColumns>
        + TryMaySetNestedColumns<Error, T::NestedDiscretionaryTriangularColumns> ,
    T::MandatoryNestedBuilders: InsertTuple<Error, Conn>,
    T::OptionalDiscretionaryNestedBuilders: InsertOptionTuple<Error, Conn>,
    T::NewRecord: TupleEqAll<EqAll: FlattenNestedTuple<Flattened: Insertable<T>>> + TypedNestedTuple<NestedTupleType=T::CompletedNewValues>,
    diesel::query_builder::InsertStatement<
        Self::Table,
        <<<T::NewRecord as TupleEqAll>::EqAll as FlattenNestedTuple>::Flattened as Insertable<T>>::Values,
    >: for<'query> diesel::query_dsl::LoadQuery<'query, Conn, <Self::Table as TableExt>::Model>,
{
    fn recursive_bundle_insert(
        mut self,
        conn: &mut Conn,
    ) -> BuilderResult<<T as TableExt>::Model, Error> {
        let mandatory_models: T::NestedMandatoryModels = self
            .nested_mandatory_associated_builders
            .insert_tuple(conn)?;
        let mandatory_primary_keys: T::NestedMandatoryPrimaryKeyTypes =
            mandatory_models.tuple_get_nested_columns();
        self.insertable_model
            .try_set_nested_columns(mandatory_primary_keys)
            .map_err(BuilderError::Validation)?;
        let discretionary_models: T::OptionalNestedDiscretionaryModels = self
            .nested_discretionary_associated_builders
            .insert_option_tuple(conn)?;
        let discretionary_primary_keys: T::OptionalNestedDiscretionaryPrimaryKeyTypes =
            discretionary_models.tuple_may_get_nested_columns();
        self.insertable_model
            .try_may_set_nested_columns(discretionary_primary_keys)
            .map_err(BuilderError::Validation)?;

        let columns = T::NewRecord::default();
        let values: T::CompletedNewValues = self
            .insertable_model
            .transpose_or(T::NewRecord::NESTED_COLUMN_NAMES)
            .map_err(|column_name| {
                BuilderError::Incomplete(IncompleteBuilderError::MissingMandatoryField(column_name))
            })?;

        Ok(diesel::insert_into(T::default())
            .values(columns.eq_all(values).flatten())
            .get_result(conn)?)
    }
}

/// Trait defining the insertion of a tuple of builders into the database.
trait InsertTuple<Error, Conn>: HasNestedTables {
    /// Insert the tuple of builders' data into the database using the provided
    /// connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if any insertion fails or if any database constraints
    /// are violated.
    fn insert_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::NestedModels, Error>;
}

impl<Err, Conn> InsertTuple<Err, Conn> for ()
where
    Conn: diesel::connection::LoadConnection,
{
    #[inline]
    fn insert_tuple(self, _conn: &mut Conn) -> BuilderResult<(), Err> {
        Ok(())
    }
}

impl<Error, Conn, T> InsertTuple<Error, Conn> for (T,)
where
    Conn: diesel::connection::LoadConnection,
    T: crate::RecursiveBuilderInsert<Error, Conn>,
{
    #[inline]
    fn insert_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::NestedModels, Error> {
        Ok((self.0.recursive_insert(conn)?,))
    }
}

impl<Error, Conn, Head, Tail> InsertTuple<Error, Conn> for (Head, Tail)
where
    Conn: diesel::connection::LoadConnection,
    Head: crate::RecursiveBuilderInsert<Error, Conn>,
    Tail: InsertTuple<Error, Conn>,
    (Head, Tail): HasNestedTables,
    Self::NestedTables: NestedTables<
        NestedModels = (
            <<Head as HasTable>::Table as TableExt>::Model,
            <Tail::NestedTables as NestedTables>::NestedModels,
        ),
    >,
{
    #[inline]
    fn insert_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::NestedModels, Error> {
        Ok((self.0.recursive_insert(conn)?, self.1.insert_tuple(conn)?))
    }
}

/// Trait defining the insertion of a tuple of optional builders into the
/// database.
trait InsertOptionTuple<Error, Conn>: HasNestedTables {
    /// Insert the tuple of optional builders' data into the database using the
    /// provided connection. If a builder is `None`, the corresponding model
    /// will also be `None`.
    ///
    /// # Arguments
    ///
    /// * `conn` - A mutable reference to the database connection.
    ///
    /// # Errors
    ///
    /// Returns an error if any insertion fails or if any database constraints
    /// are violated.
    fn insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::OptionalNestedModels, Error>;
}

impl<Err, Conn> InsertOptionTuple<Err, Conn> for () {
    #[inline]
    fn insert_option_tuple(
        self,
        _conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::OptionalNestedModels, Err> {
        Ok(())
    }
}

impl<Error, Conn, T> InsertOptionTuple<Error, Conn> for (Option<T>,)
where
    T: RecursiveBuilderInsert<Error, Conn> + HasTable,
{
    #[inline]
    fn insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::OptionalNestedModels, Error> {
        Ok((match self.0 {
            Some(builder) => Some(builder.recursive_insert(conn)?),
            None => None,
        },))
    }
}

impl<Error, Conn, Head, Tail> InsertOptionTuple<Error, Conn> for (Option<Head>, Tail)
where
    Head: RecursiveBuilderInsert<Error, Conn>,
    Tail: InsertOptionTuple<Error, Conn>,
    (Option<Head>, Tail): HasNestedTables,
    Self::NestedTables: NestedTables<
        OptionalNestedModels = (
            Option<<Head::Table as TableExt>::Model>,
            <Tail::NestedTables as NestedTables>::OptionalNestedModels,
        ),
    >,
{
    #[inline]
    fn insert_option_tuple(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::NestedTables as NestedTables>::OptionalNestedModels, Error> {
        Ok((
            match self.0 {
                Some(builder) => Some(builder.recursive_insert(conn)?),
                None => None,
            },
            self.1.insert_option_tuple(conn)?,
        ))
    }
}
