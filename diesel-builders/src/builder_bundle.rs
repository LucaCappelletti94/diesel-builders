//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::{Column, associations::HasTable};
use typed_tuple::prelude::TypedIndex;
use typed_tuple::prelude::TypedTuple;

use crate::BuilderError;
use crate::BuilderResult;
use crate::IncompleteBuilderError;
use crate::InsertableTableModel;
use crate::{
    BuildableTable, BuildableTables, ClonableTuple, Columns, DebuggableTuple, DefaultTuple,
    DiscretionarySameAsIndex, FlatInsert, HorizontalSameAsGroup, HorizontalSameAsKeys,
    MandatorySameAsIndex, MayGetColumn, NonCompositePrimaryKeyTableModels, OptionTuple,
    RecursiveInsert, RefTuple, TableAddition, TableBuilder, Tables, TransposeOptionTuple,
    TryMaySetColumns, TryMaySetDiscretionarySameAsColumn, TryMaySetDiscretionarySameAsColumns,
    TrySetColumn, TrySetColumns, TrySetMandatorySameAsColumn, TrySetMandatorySameAsColumns,
    TypedColumn,
    nested_insert::{NestedInsertOptionTuple, NestedInsertTuple},
};

/// Trait representing a Diesel table with associated mandatory and
/// discretionary triangular same-as columns.
pub trait BundlableTable: TableAddition {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularSameAsColumns: HorizontalSameAsKeys<Self, ReferencedTables: BuildableTables>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<Self, ReferencedTables: BuildableTables>;
}

/// A bundle of a table's insertable model and its associated builders.
pub struct TableBuilderBundle<T: BundlableTable> {
	/// The insertable model for the table.
	insertable_model: T::InsertableModel,
	/// The mandatory associated builders relative to triangular same-as.
	mandatory_associated_builders: <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output,
}

#[cfg(feature = "serde")]
impl<T: BundlableTable> serde::Serialize for TableBuilderBundle<T>
where
    T::InsertableModel: serde::Serialize,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: serde::Serialize,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        #[derive(serde::Serialize)]
        struct TableBuilderBundleHelper<A, B, C>
        {
            insertable_model: A,
            mandatory_associated_builders: B,
            discretionary_associated_builders: C,
        }
        let helper = TableBuilderBundleHelper {
            insertable_model: &self.insertable_model,
            mandatory_associated_builders: &self.mandatory_associated_builders,
            discretionary_associated_builders: &self.discretionary_associated_builders,
        };
        helper.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: BundlableTable> serde::Deserialize<'de> for TableBuilderBundle<T>
where
    <T as TableAddition>::InsertableModel: serde::Deserialize<'de>,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: serde::Deserialize<'de>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TableBuilderBundleHelper<A, B, C>
        {
            insertable_model: A,
            mandatory_associated_builders: B,
            discretionary_associated_builders: C,
        }

        type HelperConcrete<T> = TableBuilderBundleHelper<
            <T as TableAddition>::InsertableModel,
            <<<<T as BundlableTable>::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output,
            <<<<T as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output,
        >;

        let helper:HelperConcrete<T>  = TableBuilderBundleHelper::deserialize(deserializer)?;
        Ok(TableBuilderBundle {
            insertable_model: helper.insertable_model,
            mandatory_associated_builders: helper.mandatory_associated_builders,
            discretionary_associated_builders: helper.discretionary_associated_builders,
        })
    }
}

impl<T: BundlableTable> Clone for TableBuilderBundle<T> {
    fn clone(&self) -> Self {
        Self {
            insertable_model: self.insertable_model.clone(),
            mandatory_associated_builders: self.mandatory_associated_builders.clone_tuple(),
            discretionary_associated_builders: self.discretionary_associated_builders.clone_tuple(),
        }
    }
}

impl<T: BundlableTable> Copy for TableBuilderBundle<T>
where
    T::InsertableModel: Copy,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: Copy,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: Copy,
{
}

impl<T: BundlableTable> core::fmt::Debug for TableBuilderBundle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilderBundle")
            .field("insertable_model", &self.insertable_model)
            .field(
                "mandatory_associated_builders",
                &self.mandatory_associated_builders.debug_tuple(),
            )
            .field(
                "discretionary_associated_builders",
                &self.discretionary_associated_builders.debug_tuple(),
            )
            .finish()
    }
}

impl<T> Default for TableBuilderBundle<T>
where
    T: BundlableTable,
{
    fn default() -> Self {
        Self {
            insertable_model: Default::default(),
            mandatory_associated_builders: DefaultTuple::default_tuple(),
            discretionary_associated_builders: DefaultTuple::default_tuple(),
        }
    }
}

impl<T> HasTable for TableBuilderBundle<T>
where
    T: BundlableTable,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

/// The build-ready variant of a table builder bundle.
pub struct CompletedTableBuilderBundle<T: BundlableTable> {
	/// The insertable model for the table.
	insertable_model: T::InsertableModel,
	/// The mandatory associated builders relative to triangular same-as.
	mandatory_associated_builders: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output,
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

impl<T, C> MayGetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn,
    T::InsertableModel: MayGetColumn<C>,
{
    #[inline]
    fn may_get_column(&self) -> Option<&C::Type> {
        self.insertable_model.may_get_column()
    }
}

impl<T, C> TrySetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn<Table = T>,
    T::InsertableModel: TrySetColumn<C>,
{
    type Error = <T::InsertableModel as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        self.insertable_model.try_set_column(value)?;
        Ok(self)
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

impl<C, T> crate::SetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: crate::MandatorySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedIndex<<C as crate::MandatorySameAsIndex>::Idx, crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: typed_tuple::prelude::TypedIndex<<C as crate::MandatorySameAsIndex>::Idx, Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn set_mandatory_builder(&mut self, builder: crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>) -> &mut Self {
        self.mandatory_associated_builders.replace(Some(builder));
        self
    }
}

impl<Key> crate::TrySetMandatoryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTable,
    Key: crate::MandatorySameAsIndex,
    Key::ReferencedTable: crate::BuildableTable,
    <<<Key::Table as BundlableTable>::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
        Key::Table,
    >>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedIndex<
            <Key as crate::MandatorySameAsIndex>::Idx,
            crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>,
        >,
    <<<<Key::Table as BundlableTable>::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
        Key::Table,
    >>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output:
        typed_tuple::prelude::TypedIndex<
                <Key as crate::MandatorySameAsIndex>::Idx,
                Option<crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>>,
            >,
{
    #[inline]
    fn try_set_mandatory_builder(&mut self, builder: crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>{
        self.mandatory_associated_builders.replace(Some(builder));
        Ok(self)
    }
}

impl<C, T> crate::SetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: crate::DiscretionarySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedIndex<<C as crate::DiscretionarySameAsIndex>::Idx, crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: typed_tuple::prelude::TypedIndex<<C as crate::DiscretionarySameAsIndex>::Idx, Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn set_discretionary_builder(&mut self, builder: crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>) -> &mut Self {
        self.discretionary_associated_builders.replace(Some(builder));
        self
    }
}

impl<Key> crate::TrySetDiscretionaryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTable,
    Key: crate::DiscretionarySameAsIndex,
    Key::ReferencedTable: crate::BuildableTable,
    <<<Key::Table as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<Key::Table>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedIndex<<Key as crate::DiscretionarySameAsIndex>::Idx, crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>>,
    <<<<Key::Table as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<Key::Table>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: typed_tuple::prelude::TypedIndex<<Key as crate::DiscretionarySameAsIndex>::Idx, Option<crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error> {
        self.discretionary_associated_builders.replace(Some(builder));
        Ok(self)
    }
}

impl<Key: MandatorySameAsIndex<Table: BundlableTable, ReferencedTable: BuildableTable>, C> TrySetMandatorySameAsColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table= Key::ReferencedTable>,
    <<<<Key as Column>::Table as BundlableTable>::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<<Key as Column>::Table>>::ReferencedTables as crate::BuildableTables>::Builders: TypedIndex<<Key as MandatorySameAsIndex>::Idx, TableBuilder<<C as Column>::Table>>,
    TableBuilder<<C as Column>::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<<C as Column>::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        self.mandatory_associated_builders.map_mut(|builder| {
            builder.try_set_column(value).map(|_| ())
        })?;
        Ok(self)
    }
}

impl<Key: DiscretionarySameAsIndex<Table: BundlableTable, ReferencedTable: BuildableTable>, C> TryMaySetDiscretionarySameAsColumn<Key, C>
    for CompletedTableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table= Key::ReferencedTable>,
    <<<<<Key as Column>::Table as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<<Key as Column>::Table>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: TypedIndex<<Key as DiscretionarySameAsIndex>::Idx, Option<TableBuilder<<C as Column>::Table>>>,
    TableBuilder<<C as Column>::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<<C as Column>::Table> as TrySetColumn<C>>::Error;

    #[inline]
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: <C as TypedColumn>::Type,
    ) -> Result<&mut Self, Self::Error> {
        self.discretionary_associated_builders.map_mut(|opt_builder| {
            if let Some(builder) = opt_builder {
                builder.try_set_column(value).map(|_| ())
            } else {
                Ok(())
            }
        })?;
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
                value.mandatory_associated_builders.transpose_option()
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
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: NestedInsertOptionTuple<
        Error,
        Conn, OptionModelsTuple = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output>,
{
    fn recursive_insert(mut self, conn: &mut Conn) -> BuilderResult<<T as TableAddition>::Model, Error> {
        let mandatory_models: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models = self.mandatory_associated_builders.nested_insert_tuple(conn)?;
        let mandatory_primary_keys: <<T::MandatoryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> = mandatory_models.get_primary_keys();
        self.insertable_model.try_set_columns(mandatory_primary_keys).map_err(BuilderError::Validation)?;
        let discretionary_models: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output = self.discretionary_associated_builders.nested_insert_option_tuple(conn)?;
        let discretionary_primary_keys: <<<T::DiscretionaryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> as OptionTuple>::Output = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as NonCompositePrimaryKeyTableModels>::may_get_primary_keys(&discretionary_models);
        self.insertable_model.try_may_set_columns(discretionary_primary_keys).map_err(BuilderError::Validation)?;
        Ok(self.insertable_model.flat_insert(conn)?)
    }
}

/// Trait for n-tuples of TableBuilderBundles, providing conversion to
/// CompletedTableBuilderBundles.
pub trait BuilderBundles: DefaultTuple + ClonableTuple + DebuggableTuple {
    /// The tuple of completed builder bundles.
    type CompletedBundles;

    /// Attempt to convert all builder bundles to completed builder bundles.
    ///
    /// # Errors
    ///
    /// Returns an error if any builder bundle cannot be completed.
    fn try_complete(self) -> Result<Self::CompletedBundles, IncompleteBuilderError>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_builder_bundles]
mod impls {}
