//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::{Column, associations::HasTable};

use crate::BuilderError;
use crate::BuilderResult;
use crate::IncompleteBuilderError;
use crate::InsertableTableModel;
use crate::{
    BuildableTable, BuildableTables, Columns, DiscretionarySameAsIndex, FlatInsert,
    HorizontalSameAsGroup, HorizontalSameAsKeys, MandatorySameAsIndex, MayGetColumn,
    NonCompositePrimaryKeyTableModels, RecursiveInsert, TableAddition, TableBuilder, Tables,
    TryMaySetColumns, TryMaySetDiscretionarySameAsColumn, TryMaySetDiscretionarySameAsColumns,
    TrySetColumn, TrySetColumns, TrySetMandatorySameAsColumn, TrySetMandatorySameAsColumns,
    TypedColumn,
    nested_insert::{NestedInsertOptionTuple, NestedInsertTuple},
};
use tuplities::prelude::*;

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
	mandatory_associated_builders: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders,
}

#[cfg(feature = "serde")]
impl<T: BundlableTable> serde::Serialize for TableBuilderBundle<T>
where
    T::InsertableModel: serde::Serialize,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Serialize,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Serialize,
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
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Deserialize<'de>,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Deserialize<'de>,
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

        let helper   = TableBuilderBundleHelper::deserialize(deserializer)?;
        Ok(TableBuilderBundle {
            insertable_model: helper.insertable_model,
            mandatory_associated_builders: helper.mandatory_associated_builders,
            discretionary_associated_builders: helper.discretionary_associated_builders,
        })
    }
}

impl<T: BundlableTable> Clone for TableBuilderBundle<T>
where
    T::InsertableModel: Clone,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleClone,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleClone,
{
    fn clone(&self) -> Self {
        Self {
            insertable_model: self.insertable_model.clone(),
            mandatory_associated_builders: self.mandatory_associated_builders.tuple_clone(),
            discretionary_associated_builders: self.discretionary_associated_builders.tuple_clone(),
        }
    }
}

impl<T: BundlableTable> Copy for TableBuilderBundle<T>
where
    T::InsertableModel: Copy,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: Copy + TupleClone,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: Copy + TupleClone,
{
}

impl<T: BundlableTable> PartialEq for TableBuilderBundle<T>
where
    T::InsertableModel: PartialEq,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialEq,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.insertable_model == other.insertable_model
            && self.mandatory_associated_builders.tuple_eq(&other.mandatory_associated_builders)
            && self.discretionary_associated_builders.tuple_eq(&other.discretionary_associated_builders)
    }
}

impl<T: BundlableTable> Eq for TableBuilderBundle<T>
where
    T::InsertableModel: PartialEq + Eq,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialEq + TupleEq,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialEq + TupleEq,
{
}

impl<T: BundlableTable> std::hash::Hash for TableBuilderBundle<T>
where
    T::InsertableModel: std::hash::Hash,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleHash,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleHash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.insertable_model.hash(state);
        self.mandatory_associated_builders.tuple_hash(state);
        self.discretionary_associated_builders.tuple_hash(state);
    }
}

impl<T: BundlableTable> PartialOrd for TableBuilderBundle<T>
where
    T::InsertableModel: PartialOrd + PartialEq,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialOrd + TuplePartialEq,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialOrd + TuplePartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.insertable_model.partial_cmp(&other.insertable_model) {
            Some(std::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.mandatory_associated_builders.tuple_partial_cmp(&other.mandatory_associated_builders) {
            Some(std::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.discretionary_associated_builders.tuple_partial_cmp(&other.discretionary_associated_builders)
    }
}
impl<T: BundlableTable> Ord for TableBuilderBundle<T>
where
    T::InsertableModel: Ord + PartialOrd + Eq + PartialEq,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleOrd + TuplePartialOrd + TupleEq + TuplePartialEq,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleOrd + TuplePartialOrd + TupleEq + TuplePartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.insertable_model.cmp(&other.insertable_model) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.mandatory_associated_builders.tuple_cmp(&other.mandatory_associated_builders) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.discretionary_associated_builders.tuple_cmp(&other.discretionary_associated_builders)
    }
}

impl<T: BundlableTable> core::fmt::Debug for TableBuilderBundle<T>
where
    T::InsertableModel: core::fmt::Debug,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDebug,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDebug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilderBundle")
            .field("insertable_model", &self.insertable_model)
            .field(
                "mandatory_associated_builders",
                &self.mandatory_associated_builders.tuple_debug(),
            )
            .field(
                "discretionary_associated_builders",
                &self.discretionary_associated_builders.tuple_debug(),
            )
            .finish()
    }
}

impl<T> Default for TableBuilderBundle<T>
where
    T: BundlableTable,
    T::InsertableModel: Default,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDefault,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDefault,
{
    fn default() -> Self {
        Self {
            insertable_model: Default::default(),
            mandatory_associated_builders: TupleDefault::tuple_default(),
            discretionary_associated_builders: TupleDefault::tuple_default(),
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
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<C as crate::MandatorySameAsIndex>::Idx, Type=Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn set_mandatory_builder(&mut self, builder: crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>) -> &mut Self {
        *self.mandatory_associated_builders.tuple_index_mut() = Some(builder);
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
    >>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<
            <Key as crate::MandatorySameAsIndex>::Idx,
            Type = Option<
                crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>,
            >,
        >,
{
    #[inline]
    fn try_set_mandatory_builder(&mut self, builder: crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error>{
        *self.mandatory_associated_builders.tuple_index_mut() = Some(builder);
        Ok(self)
    }
}

impl<C, T> crate::SetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: crate::DiscretionarySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<C as crate::DiscretionarySameAsIndex>::Idx, Type=Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn set_discretionary_builder(&mut self, builder: crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>) -> &mut Self {
        *self.discretionary_associated_builders.tuple_index_mut() = Some(builder);
        self
    }
}

impl<Key> crate::TrySetDiscretionaryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTable,
    Key: crate::DiscretionarySameAsIndex,
    Key::ReferencedTable: crate::BuildableTable,
    <<<Key::Table as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<Key::Table>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<Key as crate::DiscretionarySameAsIndex>::Idx, Type=Option<crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> Result<&mut Self, <<<Self as HasTable>::Table as TableAddition>::InsertableModel as InsertableTableModel>::Error> {
        *self.discretionary_associated_builders.tuple_index_mut() = Some(builder);
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

/// Trait for n-tuples of TableBuilderBundles, providing conversion to
/// CompletedTableBuilderBundles.
pub trait BuilderBundles: TupleDefault {
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
