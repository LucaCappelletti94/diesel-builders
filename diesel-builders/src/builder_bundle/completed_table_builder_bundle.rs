//! Submodule for the completed table builder bundle and related impls.

use diesel::{Column, associations::HasTable};
use tuplities::prelude::*;

use crate::{
    BuildableTable, BuilderError, BuilderResult, BundlableTable, Columns, DiscretionarySameAsIndex,
    FlatInsert, HorizontalSameAsGroup, HorizontalSameAsKeys, IncompleteBuilderError,
    MandatorySameAsIndex, NonCompositePrimaryKeyTableModels, RecursiveInsert, TableAddition,
    TableBuilder, TableBuilderBundle, Tables, TryMaySetColumns, TryMaySetDiscretionarySameAsColumn,
    TryMaySetDiscretionarySameAsColumns, TrySetColumn, TrySetColumns, TrySetMandatorySameAsColumn,
    TrySetMandatorySameAsColumns, TypedColumn,
    nested_insert::{NestedInsertOptionTuple, NestedInsertTuple},
};

/// The build-ready variant of a table builder bundle.
pub struct CompletedTableBuilderBundle<T: BundlableTable> {
	/// The insertable model for the table.
	insertable_model: T::InsertableModel,
	/// The mandatory associated builders relative to triangular same-as.
	mandatory_associated_builders: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders,
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
    C: TypedColumn + HorizontalSameAsGroup,
    Self: TryMaySetDiscretionarySameAsColumns<
        C::Type,
        C::DiscretionaryHorizontalSameAsKeys,
        <C::DiscretionaryHorizontalSameAsKeys as HorizontalSameAsKeys<C::Table>>::FirstForeignColumns,
        Error = <T::InsertableModel as TrySetColumn<C>>::Error,
    >,
    Self: TrySetMandatorySameAsColumns<
        C::Type,
        C::MandatoryHorizontalSameAsKeys,
        <C::MandatoryHorizontalSameAsKeys as HorizontalSameAsKeys<C::Table>>::FirstForeignColumns,
        Error = <T::InsertableModel as TrySetColumn<C>>::Error,
    >,
    T::InsertableModel: TrySetColumn<C>,
{
    type Error = <T::InsertableModel as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error>
    {
        self.try_may_set_discretionary_same_as_columns(&value)?;
        self.try_set_mandatory_same_as_columns(&value)?;
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<Key: MandatorySameAsIndex<Table: BundlableTable, ReferencedTable: BuildableTable>, C> TrySetMandatorySameAsColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table= Key::ReferencedTable>,
    <<<<Key as Column>::Table as BundlableTable>::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<<Key as Column>::Table>>::ReferencedTables as crate::BuildableTables>::Builders: TupleIndexMut<<Key as MandatorySameAsIndex>::Idx, Type=TableBuilder<<C as Column>::Table>>,
    TableBuilder<<C as Column>::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<<C as Column>::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        self.mandatory_associated_builders.tuple_index_mut().try_set_column(value)?;
        Ok(self)
    }
}

impl<Key: DiscretionarySameAsIndex<Table: BundlableTable, ReferencedTable: BuildableTable>, C> TryMaySetDiscretionarySameAsColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table= Key::ReferencedTable>,
    <<<<Key as Column>::Table as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<<Key as Column>::Table>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<Key as DiscretionarySameAsIndex>::Idx, Type=Option<TableBuilder<<C as Column>::Table>>>,
    TableBuilder<<C as Column>::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<<C as Column>::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        if let Some(builder) = self.discretionary_associated_builders.tuple_index_mut().as_mut() {
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

impl<T, Error, Conn> RecursiveInsert<Error, Conn> for CompletedTableBuilderBundle<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BundlableTable,
    T::InsertableModel: FlatInsert<Conn>,
    T::InsertableModel: TrySetColumns<Error, T::MandatoryTriangularSameAsColumns>,
    T::InsertableModel: TryMaySetColumns<Error, T::DiscretionaryTriangularSameAsColumns>,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: NestedInsertTuple<
        Error,
    Conn, ModelsTuple = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models>,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: NestedInsertOptionTuple<
        Error,
        Conn, OptionModelsTuple = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as IntoTupleOption>::IntoOptions>,
{
    fn recursive_insert(mut self, conn: &mut Conn) -> BuilderResult<<T as TableAddition>::Model, Error> {
        let mandatory_models: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models = self.mandatory_associated_builders.nested_insert_tuple(conn)?;
        let mandatory_primary_keys: <<T::MandatoryTriangularSameAsColumns as Columns>::Types as TupleRef>::Ref<'_> = mandatory_models.get_primary_keys();
        self.insertable_model.try_set_columns(mandatory_primary_keys).map_err(BuilderError::Validation)?;
        let discretionary_models: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as IntoTupleOption>::IntoOptions = self.discretionary_associated_builders.nested_insert_option_tuple(conn)?;
        let discretionary_primary_keys = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as NonCompositePrimaryKeyTableModels>::may_get_primary_keys(&discretionary_models);
        self.insertable_model.try_may_set_columns(discretionary_primary_keys).map_err(BuilderError::Validation)?;
        Ok(self.insertable_model.flat_insert(conn)?)
    }
}
