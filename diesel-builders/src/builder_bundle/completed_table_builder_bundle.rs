//! Submodule for the completed table builder bundle and related impls.

use diesel::{Column, associations::HasTable};
use tuplities::prelude::*;

use crate::{
    BuildableTable, BuilderError, BuilderResult, BundlableTable, DiscretionarySameAsIndex,
    FlatInsert, HasTableAddition, IncompleteBuilderError, MandatorySameAsIndex, TableAddition,
    TableBuilder, TableBuilderBundle, TryMaySetColumns, TryMaySetDiscretionarySameAsColumn,
    TryMaySetDiscretionarySameAsColumns, TrySetColumn, TrySetColumns, TrySetMandatorySameAsColumn,
    TrySetMandatorySameAsColumns, TupleGetColumns, TupleMayGetColumns, Typed, TypedColumn,
    builder_bundle::BundlableTableExt, horizontal_same_as_group::HorizontalSameAsGroupExt,
    tables::TableModels,
};

/// The build-ready variant of a table builder bundle.
pub struct CompletedTableBuilderBundle<T: BundlableTableExt> {
    /// The insertable model for the table.
    insertable_model: T::InsertableModel,
    /// The mandatory associated builders relative to triangular same-as.
    mandatory_associated_builders: T::MandatoryBuilders,
    /// The discretionary associated builders relative to triangular same-as.
    discretionary_associated_builders: T::OptionalDiscretionaryBuilders,
}

impl<T> HasTable for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, C> TrySetColumn<C> for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn + HorizontalSameAsGroupExt,
    Self: TryMaySetDiscretionarySameAsColumns<
            C::Type,
            <T::InsertableModel as TrySetColumn<C>>::Error,
            C::DiscretionaryHorizontalSameAsKeys,
            C::DiscretionaryForeignColumns,
        >,
    Self: TrySetMandatorySameAsColumns<
            C::Type,
            <T::InsertableModel as TrySetColumn<C>>::Error,
            C::MandatoryHorizontalSameAsKeys,
            C::MandatoryForeignColumns,
        >,
    T::InsertableModel: TrySetColumn<C>,
{
    type Error = <T::InsertableModel as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.try_may_set_discretionary_same_as_columns(&value)?;
        self.try_set_mandatory_same_as_columns(&value)?;
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<Key: MandatorySameAsIndex<Table: BundlableTable, ReferencedTable: BuildableTable>, C>
    TrySetMandatorySameAsColumn<Key, C> for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <<Key as Column>::Table as BundlableTableExt>::MandatoryBuilders:
        TupleIndexMut<Key::Idx, Element = TableBuilder<<C as Column>::Table>>,
    TableBuilder<<C as Column>::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<<C as Column>::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: <C as Typed>::Type,
    ) -> Result<&mut Self, Self::Error> {
        self.mandatory_associated_builders
            .tuple_index_mut()
            .try_set_column(value)?;
        Ok(self)
    }
}

impl<Key: DiscretionarySameAsIndex<Table: BundlableTable, ReferencedTable: BuildableTable>, C>
    TryMaySetDiscretionarySameAsColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <<Key as Column>::Table as BundlableTableExt>::OptionalDiscretionaryBuilders:
        TupleIndexMut<Key::Idx, Element = Option<TableBuilder<<C as Column>::Table>>>,
    TableBuilder<<C as Column>::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<<C as Column>::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: <C as Typed>::Type,
    ) -> Result<&mut Self, Self::Error> {
        if let Some(builder) = self
            .discretionary_associated_builders
            .tuple_index_mut()
            .as_mut()
        {
            builder.try_set_column(value)?;
        }
        Ok(self)
    }
}

impl<T> TryFrom<TableBuilderBundle<T>> for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
{
    type Error = IncompleteBuilderError;

    fn try_from(
        value: TableBuilderBundle<T>,
    ) -> Result<CompletedTableBuilderBundle<T>, Self::Error> {
        Ok(CompletedTableBuilderBundle {
            insertable_model: value.insertable_model,
            mandatory_associated_builders: if let Some(mandatory_associated_builders) =
                value.mandatory_associated_builders.transpose()
            {
                mandatory_associated_builders
            } else {
                return Err(IncompleteBuilderError::MissingMandatoryTriangularFields);
            },
            discretionary_associated_builders: value.discretionary_associated_builders,
        })
    }
}

/// Trait defining the insertion of a builder into the database.
pub trait RecursiveBundleInsert<Error, Conn>: HasTableAddition {
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
    ) -> BuilderResult<<<Self as HasTable>::Table as TableAddition>::Model, Error>;
}

impl<T, Error, Conn> RecursiveBundleInsert<Error, Conn> for CompletedTableBuilderBundle<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BundlableTableExt,
    T::InsertableModel: FlatInsert<Conn>,
    T::InsertableModel: TrySetColumns<Error, T::MandatoryTriangularSameAsColumns>,
    T::InsertableModel: TryMaySetColumns<Error, T::DiscretionaryTriangularSameAsColumns>,
    T::MandatoryBuilders: InsertTuple<Error, Conn, ModelsTuple = T::MandatoryModels>,
    T::OptionalDiscretionaryBuilders:
        InsertOptionTuple<Error, Conn, OptionModelsTuple = T::OptionalDiscretionaryModels>,
{
    fn recursive_bundle_insert(
        mut self,
        conn: &mut Conn,
    ) -> BuilderResult<<T as TableAddition>::Model, Error> {
        let mandatory_models: T::MandatoryModels =
            self.mandatory_associated_builders.insert_tuple(conn)?;
        let mandatory_primary_keys: <<T::MandatoryTriangularSameAsColumns as Typed>::Type as TupleRef>::Ref<'_> = mandatory_models.tuple_get_columns();
        self.insertable_model
            .try_set_columns(mandatory_primary_keys)
            .map_err(BuilderError::Validation)?;
        let discretionary_models: T::OptionalDiscretionaryModels = self
            .discretionary_associated_builders
            .insert_option_tuple(conn)?;
        let discretionary_primary_keys = discretionary_models.tuple_may_get_columns();
        self.insertable_model
            .try_may_set_columns(discretionary_primary_keys)
            .map_err(BuilderError::Validation)?;
        Ok(self.insertable_model.flat_insert(conn)?)
    }
}

#[diesel_builders_macros::impl_insert_tuple]
/// Trait defining the insertion of a tuple of builders into the database.
trait InsertTuple<Error, Conn> {
    /// The type of the models associated with the builders in the tuple.
    type ModelsTuple: TableModels;

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
    fn insert_tuple(self, conn: &mut Conn) -> BuilderResult<Self::ModelsTuple, Error>;
}

#[diesel_builders_macros::impl_insert_option_tuple]
/// Trait defining the insertion of a tuple of optional builders into the
/// database.
trait InsertOptionTuple<Error, Conn> {
    /// The type of the optional models associated with the builders in the
    /// tuple.
    type OptionModelsTuple;

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
    fn insert_option_tuple(self, conn: &mut Conn) -> BuilderResult<Self::OptionModelsTuple, Error>;
}
