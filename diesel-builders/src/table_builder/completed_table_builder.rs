//! Submodule for the completed table builder and related impls.

use std::ops::Sub;

use diesel::{Table, associations::HasTable};
use tuplities::prelude::{
    FlattenNestedTuple, NestTuple, NestedTupleIndex, NestedTupleIndexMut, NestedTupleReplicate,
    NestedTupleTryFrom,
};

use crate::{
    AncestorOfIndex, BuildableTable, BuilderError, BuilderResult, BundlableTable,
    CompletedTableBuilderBundle, DescendantOf, DescendantWithSelf, EitherValidationError,
    GetNestedColumns, HasNestedTables, HasTableExt, IncompleteBuilderError, Insert, NestedColumns,
    NestedTables, TableBuilder, TableExt, TrySetColumn, TypedColumn, TypedNestedTuple,
    TypedNestedTupleCollection, ValidateColumn, VerticalSameAsGroup,
    columns::{NestedColumnsCollection, NonEmptyNestedProjection},
    insert_transaction::{
        InsertBuilder, InsertBundle, SetTableKeyColumns, SetTableKeyColumnsCollection,
    },
};

/// A completed builder for creating insertable models for a Diesel table and
/// its ancestors.
pub struct RecursiveTableBuilder<T: diesel::Table, Depth, NestedBundles> {
    /// The insertable models for the table and its ancestors.
    nested_bundles: NestedBundles,
    /// Marker for the table and depth.
    _markers: std::marker::PhantomData<(T, Depth)>,
}

impl<T: diesel::Table, Depth, NestedBundles> RecursiveTableBuilder<T, Depth, NestedBundles> {
    /// Create a new `RecursiveTableBuilder` from the provided nested builder
    /// bundles.
    ///
    /// This is a private convenience constructor used during `TryFrom`
    /// conversions when assembling a builder from its nested parts.
    fn from_nested_bundles(nested_bundles: NestedBundles) -> Self {
        RecursiveTableBuilder { nested_bundles, _markers: std::marker::PhantomData }
    }
}

/// Trait defining the insertion of a builder into the database.
pub trait RecursiveBuilderInsert<Error, Conn>: HasTableExt {
    /// The nested model types returned after insertion.
    type NestedModels;

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
    /// Insert a record with a triangular dependency, then verify the keys.
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::RecursiveBuilderInsert;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let builder = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short");
    /// let child = builder.recursive_insert(&mut conn)?;
    /// let mandatory: Side = child.mandatory(&mut conn)?;
    /// assert_eq!(mandatory.get_column::<sides::parent_id>(), child.get_column::<children::id>());
    /// # Ok(())
    /// # }
    /// ```
    fn recursive_insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Error>;

    /// Insert the builder's data into the database using the provided
    /// connection, returning a nested tuple with all of the inserted models.
    ///
    /// # Errors
    ///
    /// Returns an error if the insertion fails or if any database constraints
    /// are violated.
    ///
    /// # Examples
    ///
    /// Insert a record and read back the nested model tuple.
    ///
    /// ```rust
    /// # include!("../doctest_setup.rs");
    /// # use schema::*;
    /// # use diesel_builders::RecursiveBuilderInsert;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut conn = connection()?;
    /// let builder = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short");
    /// let nested = builder.recursive_insert_nested(&mut conn)?;
    /// assert_eq!(nested.child_label(), "short");
    /// assert_eq!(children::table.count().get_result::<i64>(&mut conn)?, 1);
    /// # Ok(())
    /// # }
    /// ```
    fn recursive_insert_nested(self, conn: &mut Conn) -> BuilderResult<Self::NestedModels, Error>;
}

impl<T, Conn> InsertBuilder<Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    T::NestedAncestorBuilders: NestTuple,
    Self: HasTable<Table = T>,
    RecursiveTableBuilder<T, typenum::U0, T::NestedCompletedAncestorBuilders>:
        TryFrom<Self, Error = IncompleteBuilderError>
            + InsertBuilder<
                Conn,
                Table = T,
                NestedModels = <<T as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
            >
            + HasTable<Table = T>,
{
    type NestedModels =
        <<T as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels;
    type ValidationError = <
        RecursiveTableBuilder<T, typenum::U0, T::NestedCompletedAncestorBuilders> as InsertBuilder<
            Conn,
        >
    >::ValidationError;

    #[inline]
    fn insert_builder(self, conn: &mut Conn) -> BuilderResult<T::Model, Self::ValidationError> {
        let completed_builder: RecursiveTableBuilder<
            T,
            typenum::U0,
            T::NestedCompletedAncestorBuilders,
        > = self.try_into()?;
        completed_builder.insert_builder(conn)
    }

    fn insert_builder_nested(self, conn: &mut Conn) -> BuilderResult<Self::NestedModels, Self::ValidationError> {
        let completed_builder: RecursiveTableBuilder<
            T,
            typenum::U0,
            T::NestedCompletedAncestorBuilders,
        > = self.try_into()?;
        completed_builder.insert_builder_nested(conn)
    }
}

impl<T: BuildableTable + DescendantWithSelf, Conn, Error> Insert<Conn> for TableBuilder<T>
where
    Conn: diesel::Connection,
    Self: InsertBuilder<
        Conn,
        Table = T,
        ValidationError = Error,
        NestedModels = <<Self::Table as DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels,
    > + HasTable<Table = T>,
{
    type ValidationError = Error;

    #[inline]
    fn insert(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Self::Table as TableExt>::Model, Self::ValidationError> {
        conn.transaction(|conn| self.insert_builder(conn))
    }

    fn insert_nested(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<<Self::Table as crate::DescendantWithSelf>::NestedAncestorsWithSelf as NestedTables>::NestedModels, Self::ValidationError> {
        conn.transaction(|conn| self.insert_builder_nested(conn))
    }
}

impl<T: Table + Default, Depth, Bundles> HasTable for RecursiveTableBuilder<T, Depth, Bundles> {
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, C, Depth, Bundles> ValidateColumn<C> for RecursiveTableBuilder<T, Depth, Bundles>
where
    Bundles: NestedTupleIndex<
            <<C::Table as AncestorOfIndex<T>>::Idx as Sub<Depth>>::Output,
            Element = CompletedTableBuilderBundle<C::Table>,
        >,
    T: BuildableTable + DescendantOf<C::Table>,
    C: TypedColumn,
    C::Table: AncestorOfIndex<T, Idx: Sub<Depth>> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: ValidateColumn<C>,
{
    type Error = <CompletedTableBuilderBundle<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn validate_column(value: &C::ValueType) -> Result<(), Self::Error> {
        <CompletedTableBuilderBundle<C::Table> as ValidateColumn<C>>::validate_column(value)
    }
}

impl<T, C, Depth, Bundles> crate::mutation::PrepareColumn<C>
    for RecursiveTableBuilder<T, Depth, Bundles>
where
    Bundles: NestedTupleIndexMut<
            <<C::Table as AncestorOfIndex<T>>::Idx as Sub<Depth>>::Output,
            Element = CompletedTableBuilderBundle<C::Table>,
        >,
    T: BuildableTable + DescendantOf<C::Table>,
    C: VerticalSameAsGroup,
    C::Table: AncestorOfIndex<T, Idx: Sub<Depth>> + BundlableTable,
    CompletedTableBuilderBundle<C::Table>: crate::mutation::PrepareColumn<C>,
    Self: crate::mutation::PrepareHomogeneous<
            Self::Error,
            C::ValueType,
            C::VerticalSameAsNestedColumns,
        >,
{
    type Prepared = super::PreparedBuilderColumn<
        <CompletedTableBuilderBundle<C::Table> as crate::mutation::PrepareColumn<C>>::Prepared,
        <Self as crate::mutation::PrepareHomogeneous<
            Self::Error,
            C::ValueType,
            C::VerticalSameAsNestedColumns,
        >>::Prepared,
    >;

    fn prepare_column(
        &self,
        value: C::ColumnType,
        context: &crate::mutation::MutationContext,
    ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)> {
        use crate::mutation::{ColumnInput, PrepareColumn, PrepareHomogeneous};
        let own = <CompletedTableBuilderBundle<C::Table> as PrepareColumn<C>>::prepare_column(
            self.nested_bundles.nested_index(),
            value,
            context,
        )?;
        match self.prepare_homogeneous(own.value(), context) {
            Ok(vertical) => Ok(super::PreparedBuilderColumn { own, vertical }),
            Err(error) => Err((own.into_value(), error)),
        }
    }

    fn apply_column(&mut self, prepared: Self::Prepared) {
        use crate::mutation::{PrepareColumn, PrepareHomogeneous};
        self.apply_homogeneous(prepared.vertical);
        <CompletedTableBuilderBundle<C::Table> as PrepareColumn<C>>::apply_column(
            self.nested_bundles.nested_index_mut(),
            prepared.own,
        );
    }
}

impl<T: diesel::Table, C: TypedColumn, Depth, Bundles> TrySetColumn<C>
    for RecursiveTableBuilder<T, Depth, Bundles>
where
    Self: crate::mutation::PrepareColumn<C>,
{
    #[inline]
    fn try_set_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        let prepared = <Self as crate::mutation::PrepareColumn<C>>::prepare_column(
            self,
            value.into(),
            &crate::mutation::MutationContext::default(),
        )
        .map_err(|(_, error)| error)?;
        <Self as crate::mutation::PrepareColumn<C>>::apply_column(self, prepared);
        Ok(self)
    }
}

// Base case: single element nested tuple
impl<T: diesel::Table, Depth, Conn, Head> InsertBuilder<Conn>
    for RecursiveTableBuilder<T, Depth, (Head,)>
where
    Conn: diesel::connection::LoadConnection,
    Head: InsertBundle<Conn>,
    Self: HasTableExt<Table = Head::Table>,
{
    type NestedModels = (<Head::Table as TableExt>::Model,);
    type ValidationError = <Head as InsertBundle<Conn>>::ValidationError;

    #[inline]
    fn insert_builder(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<<Head::Table as TableExt>::Model, Self::ValidationError> {
        self.nested_bundles.0.insert_bundle(conn)
    }

    fn insert_builder_nested(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<Self::NestedModels, Self::ValidationError> {
        self.nested_bundles.0.insert_bundle(conn).map(|model| (model,))
    }
}

// Recursive case: nested 2-tuple (Head, Tail) where Tail is itself a nested
// tuple
impl<T, Depth, Conn, Head, Tail> InsertBuilder<Conn>
    for RecursiveTableBuilder<T, Depth, (Head, Tail)>
where
    T: TableExt,
    Conn: diesel::connection::LoadConnection,
    Head: InsertBundle<Conn> + HasTable,
    Tail: FlattenNestedTuple + HasNestedTables,
    <Head::Table as TableExt>::Model:
        GetNestedColumns<<Head::Table as TableExt>::NestedPrimaryKeyColumns>,
    // Tail: HasNestedTables (moved into the combined bound above)
    Depth: core::ops::Add<typenum::U1>,
    RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail>:
        InsertBuilder<
            Conn,
            Table = T,
            NestedModels = <Tail::NestedTables as NestedTables>::NestedModels,
        >
            + SetTableKeyColumnsCollection<
                <Tail::NestedTables as NestedTables>::NestedPrimaryKeyColumnsCollection,
            >,
    <Tail::NestedTables as NestedTables>::NestedPrimaryKeyColumnsCollection: NestedColumnsCollection<
        NestedCollectionType: NestedTupleReplicate<
            <<Head::Table as TableExt>::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleColumnType,
        >,
    >,
{
    type NestedModels = (
        <Head::Table as TableExt>::Model,
        <Tail::NestedTables as NestedTables>::NestedModels,
    );
    type ValidationError = EitherValidationError<
        <Head as InsertBundle<Conn>>::ValidationError,
        EitherValidationError<
            <
                RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail> as SetTableKeyColumnsCollection<
                    <Tail::NestedTables as NestedTables>::NestedPrimaryKeyColumnsCollection,
                >
            >::Error,
            <
                RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail> as InsertBuilder<
                    Conn,
                >
            >::ValidationError,
        >,
    >;

    #[inline]
    fn insert_builder(
        self,
        conn: &mut Conn,
    ) -> BuilderResult<T::Model, Self::ValidationError> {
        // Insert the first table and get its model (with primary keys)
        let first = self.nested_bundles.0;
        let model: <Head::Table as TableExt>::Model = first
            .insert_bundle(conn)
            .map_err(|error| error.map_validation(EitherValidationError::Left))?;
        // Extract primary keys and set them in the tail builder
        let mut tail_builder: RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail> =
            RecursiveTableBuilder::from_nested_bundles(self.nested_bundles.1);
        tail_builder
            .set_table_key_columns_collection(
                <<<Tail::NestedTables as NestedTables>::NestedPrimaryKeyColumnsCollection as TypedNestedTupleCollection>::NestedCollectionType as NestedTupleReplicate<
                    <<Head::Table as TableExt>::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleColumnType,
                >>::nested_tuple_replicate(model.get_nested_columns()),
            )
            .map_err(|error| {
                BuilderError::Validation(EitherValidationError::Right(EitherValidationError::Left(
                    error,
                )))
            })?;
        // Recursively insert the tail
        tail_builder
            .insert_builder(conn)
            .map_err(|error| error.map_validation(|error| {
                EitherValidationError::Right(EitherValidationError::Right(error))
            }))
    }

    fn insert_builder_nested(self, conn: &mut Conn) -> BuilderResult<Self::NestedModels, Self::ValidationError> {
        // Insert the first table and get its model (with primary keys)
        let first = self.nested_bundles.0;
        let model: <Head::Table as TableExt>::Model = first
            .insert_bundle(conn)
            .map_err(|error| error.map_validation(EitherValidationError::Left))?;
        // Extract primary keys and set them in the tail builder
        let mut tail_builder: RecursiveTableBuilder<T, typenum::Sum<Depth, typenum::U1>, Tail> =
            RecursiveTableBuilder::from_nested_bundles(self.nested_bundles.1);
        tail_builder
            .set_table_key_columns_collection(
                <<<Tail::NestedTables as NestedTables>::NestedPrimaryKeyColumnsCollection as TypedNestedTupleCollection>::NestedCollectionType as NestedTupleReplicate<
                    <<Head::Table as TableExt>::NestedPrimaryKeyColumns as TypedNestedTuple>::NestedTupleColumnType,
                >>::nested_tuple_replicate(model.get_nested_columns()),
            )
            .map_err(|error| {
                BuilderError::Validation(EitherValidationError::Right(EitherValidationError::Left(
                    error,
                )))
            })?;
        // Recursively insert the tail
        Ok((
            model,
            tail_builder
                .insert_builder_nested(conn)
                .map_err(|error| error.map_validation(|error| {
                    EitherValidationError::Right(EitherValidationError::Right(error))
                }))?,
        ))
    }
}

impl<T> TryFrom<TableBuilder<T>>
    for RecursiveTableBuilder<T, typenum::U0, T::NestedCompletedAncestorBuilders>
where
    T: BuildableTable,
{
    type Error = IncompleteBuilderError;

    #[inline]
    fn try_from(value: TableBuilder<T>) -> Result<Self, Self::Error> {
        Ok(RecursiveTableBuilder::from_nested_bundles(
            <T::NestedCompletedAncestorBuilders as NestedTupleTryFrom<
                T::NestedAncestorBuilders,
                IncompleteBuilderError,
            >>::nested_tuple_try_from(value.bundles)?,
        ))
    }
}

impl<T: diesel::Table, C, Depth, Bundles> SetTableKeyColumns<(C,)>
    for RecursiveTableBuilder<T, Depth, Bundles>
where
    C: TypedColumn<Table: TableExt>,
    Self: TrySetColumn<C>,
{
    type Error = <Self as ValidateColumn<C>>::Error;

    #[inline]
    fn set_table_key_columns(
        &mut self,
        values: (C::ColumnType,),
    ) -> Result<&mut Self, Self::Error> {
        self.try_set_column(values.0)?;
        Ok(self)
    }
}

impl<T: diesel::Table, C, Tail, Depth, Bundles> SetTableKeyColumns<(C, Tail)>
    for RecursiveTableBuilder<T, Depth, Bundles>
where
    C: TypedColumn,
    Tail: NestedColumns,
    (C, Tail): NestedColumns<NestedTupleColumnType = (C::ColumnType, Tail::NestedTupleColumnType)>,
    Self: TrySetColumn<C> + SetTableKeyColumns<Tail>,
{
    type Error = EitherValidationError<
        <Self as ValidateColumn<C>>::Error,
        <Self as SetTableKeyColumns<Tail>>::Error,
    >;

    #[inline]
    fn set_table_key_columns(
        &mut self,
        values: (C::ColumnType, Tail::NestedTupleColumnType),
    ) -> Result<&mut Self, Self::Error> {
        self.try_set_column(values.0).map_err(EitherValidationError::Left)?;
        self.set_table_key_columns(values.1).map_err(EitherValidationError::Right)?;
        Ok(self)
    }
}

impl<T: diesel::Table, C, Depth, Bundles> SetTableKeyColumnsCollection<(C,)>
    for RecursiveTableBuilder<T, Depth, Bundles>
where
    C: NonEmptyNestedProjection,
    Self: SetTableKeyColumns<C>,
{
    type Error = <Self as SetTableKeyColumns<C>>::Error;

    #[inline]
    fn set_table_key_columns_collection(
        &mut self,
        values: (C::NestedTupleColumnType,),
    ) -> Result<&mut Self, Self::Error> {
        self.set_table_key_columns(values.0)
    }
}

impl<T: diesel::Table, C, Tail, Depth, Bundles> SetTableKeyColumnsCollection<(C, Tail)>
    for RecursiveTableBuilder<T, Depth, Bundles>
where
    C: NonEmptyNestedProjection,
    Tail: NestedColumnsCollection,
    (C, Tail): TypedNestedTupleCollection<
        NestedCollectionType = (
            C::NestedTupleColumnType,
            <Tail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
    >,
    Self: SetTableKeyColumns<C> + SetTableKeyColumnsCollection<Tail>,
{
    type Error = EitherValidationError<
        <Self as SetTableKeyColumns<C>>::Error,
        <Self as SetTableKeyColumnsCollection<Tail>>::Error,
    >;

    #[inline]
    fn set_table_key_columns_collection(
        &mut self,
        values: (
            C::NestedTupleColumnType,
            <Tail as TypedNestedTupleCollection>::NestedCollectionType,
        ),
    ) -> Result<&mut Self, Self::Error> {
        self.set_table_key_columns(values.0).map_err(EitherValidationError::Left)?;
        self.set_table_key_columns_collection(values.1).map_err(EitherValidationError::Right)?;
        Ok(self)
    }
}
