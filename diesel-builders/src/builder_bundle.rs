//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table record new values and its mandatory and discretionary associated
//! builders.

use std::borrow::Borrow;

use diesel::Column;
use diesel::associations::HasTable;

mod completed_table_builder_bundle;
mod serde;
pub use completed_table_builder_bundle::CompletedTableBuilderBundle;
pub use completed_table_builder_bundle::RecursiveBundleInsert;

use crate::OptionalRef;
use crate::SetColumn;
use crate::SetDiscretionarySameAsNestedColumns;
use crate::SetMandatorySameAsNestedColumns;
use crate::TrySetDiscretionarySameAsColumn;
use crate::TrySetDiscretionarySameAsNestedColumns;
use crate::TrySetMandatorySameAsColumn;
use crate::TrySetMandatorySameAsNestedColumns;
use crate::ValidateColumn;
use crate::columns::NestedColumns;
use crate::horizontal_same_as_group::HorizontalSameAsGroupExt;
use crate::tables::NonCompositePrimaryKeyNestedTables;
use crate::{
    BuildableTable, Columns, DiscretionarySameAsIndex, HasNestedTables, HorizontalNestedKeys,
    MandatorySameAsIndex, NestedBuildableTables, NestedTableModels, NestedTables,
    SetDiscretionaryBuilder, SetMandatoryBuilder, TableBuilder, TrySetDiscretionaryBuilder,
    TrySetMandatoryBuilder, TupleMayGetNestedColumns, TypedNestedTuple,
};
use crate::{MayGetColumn, TableExt, TrySetColumn, TupleGetNestedColumns, TypedColumn};
use tuplities::prelude::*;

/// Trait representing a Diesel table with associated mandatory and
/// discretionary triangular same-as columns.
pub trait BundlableTable: Sized {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularColumns: Columns<Nested: HorizontalNestedKeys<Self>>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularColumns: Columns<Nested: HorizontalNestedKeys<Self>>;
}

/// Extension trait for `BundlableTable`.
pub trait BundlableTableExt:
    BundlableTable + TableExt<NewValues: NestedTupleOption<Transposed = Self::CompletedNewValues>>
{
    /// The completed new values ready to be inserted for the table.
    type CompletedNewValues: FlattenNestedTuple
        + IntoNestedTupleOption<IntoOptions = Self::NewValues>;
    /// Nested mandatory triangular same-as columns.
    type NestedMandatoryTriangularColumns: NestedColumns<
        NestedTupleColumnType = Self::NestedMandatoryPrimaryKeyTypes,
    >;
    /// Nested mandatory tables.
    type NestedMandatoryTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns = Self::NestedMandatoryPrimaryKeys,
            NestedModels = Self::NestedMandatoryModels,
        > + NestedBuildableTables;
    /// Nested discretionary triangular same-as columns.
    type NestedDiscretionaryTriangularColumns: NestedColumns<
        NestedTupleColumnType = Self::NestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested discretionary tables.
    type NestedDiscretionaryTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns = Self::NestedDiscretionaryPrimaryKeys,
            NestedModels = Self::NestedDiscretionaryModels,
            OptionalNestedModels = Self::OptionalNestedDiscretionaryModels,
        > + NestedBuildableTables;
    /// Nested mandatory foreign primary keys.
    type NestedMandatoryPrimaryKeys: NestedColumns<
        NestedTupleColumnType = Self::NestedMandatoryPrimaryKeyTypes,
    >;
    /// Nested mandatory foreign primary keys types.
    type NestedMandatoryPrimaryKeyTypes;
    /// Nested discretionary foreign primary keys.
    type NestedDiscretionaryPrimaryKeys: NestedColumns<
        NestedTupleColumnType = Self::NestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested discretionary foreign primary key types.
    type NestedDiscretionaryPrimaryKeyTypes: IntoNestedTupleOption<
        IntoOptions = Self::OptionalNestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested optional discretionary foreign primary key types.
    type OptionalNestedDiscretionaryPrimaryKeyTypes;
    /// Builders for the mandatory associated tables.
    type MandatoryNestedBuilders: IntoNestedTupleOption<IntoOptions = Self::OptionalMandatoryNestedBuilders>
        + HasNestedTables<NestedTables = Self::NestedMandatoryTables>;
    /// Optional builders for the mandatory associated tables.
    type OptionalMandatoryNestedBuilders: NestedTupleOptionWith<
        &'static str,
        SameDepth = <Self::NestedMandatoryTriangularColumns as NestedColumns>::NestedColumnNames,
        Transposed = Self::MandatoryNestedBuilders
    >
        + HasNestedTables<NestedTables = Self::NestedMandatoryTables>;
    /// Builders for the discretionary associated tables.
    type DiscretionaryNestedBuilders: IntoNestedTupleOption<IntoOptions = Self::OptionalDiscretionaryNestedBuilders>
        + HasNestedTables<NestedTables = Self::NestedDiscretionaryTables>;
    /// Optional builders for the discretionary associated tables.
    type OptionalDiscretionaryNestedBuilders: NestedTupleOption<Transposed = Self::DiscretionaryNestedBuilders>
        + HasNestedTables<NestedTables = Self::NestedDiscretionaryTables>;
    /// The nested mandatory models.
    type NestedMandatoryModels: NestedTableModels<NestedTables = Self::NestedMandatoryTables>
        + TupleGetNestedColumns<Self::NestedMandatoryPrimaryKeys>;
    /// The nested discretionary models.
    type NestedDiscretionaryModels: NestedTableModels<
            IntoOptions = Self::OptionalNestedDiscretionaryModels,
            NestedTables = Self::NestedDiscretionaryTables,
        > + TupleGetNestedColumns<Self::NestedDiscretionaryPrimaryKeys>;
    /// The nested optional discretionary models.
    type OptionalNestedDiscretionaryModels: TupleMayGetNestedColumns<
        Self::NestedDiscretionaryPrimaryKeys,
    >;
}

impl<T> BundlableTableExt for T
where
    T: BundlableTable + TableExt,
{
    type CompletedNewValues = <T::NewRecord as TypedNestedTuple>::NestedTupleColumnType;
    type NestedMandatoryTriangularColumns = <T::MandatoryTriangularColumns as NestTuple>::Nested;
    type NestedMandatoryTables =
        <Self::NestedMandatoryTriangularColumns as HorizontalNestedKeys<T>>::NestedReferencedTables;
    type NestedDiscretionaryTriangularColumns =
        <T::DiscretionaryTriangularColumns as NestTuple>::Nested;
    type NestedDiscretionaryTables =
        <Self::NestedDiscretionaryTriangularColumns as HorizontalNestedKeys<
            T,
        >>::NestedReferencedTables;
    type NestedMandatoryPrimaryKeys =
        <Self::NestedMandatoryTables as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns;
    type NestedMandatoryPrimaryKeyTypes =
        <Self::NestedMandatoryPrimaryKeys as TypedNestedTuple>::NestedTupleColumnType;
    type NestedDiscretionaryPrimaryKeys =
        <Self::NestedDiscretionaryTables as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns;
    type NestedDiscretionaryPrimaryKeyTypes =
        <Self::NestedDiscretionaryPrimaryKeys as TypedNestedTuple>::NestedTupleColumnType;
    type OptionalNestedDiscretionaryPrimaryKeyTypes =
        <Self::NestedDiscretionaryPrimaryKeyTypes as IntoNestedTupleOption>::IntoOptions;
    type MandatoryNestedBuilders =
        <Self::NestedMandatoryTables as NestedBuildableTables>::NestedBuilders;
    type OptionalMandatoryNestedBuilders =
        <Self::MandatoryNestedBuilders as IntoNestedTupleOption>::IntoOptions;
    type DiscretionaryNestedBuilders =
        <Self::NestedDiscretionaryTables as NestedBuildableTables>::NestedBuilders;
    type OptionalDiscretionaryNestedBuilders =
        <Self::DiscretionaryNestedBuilders as IntoNestedTupleOption>::IntoOptions;
    type NestedMandatoryModels = <Self::NestedMandatoryTables as NestedTables>::NestedModels;
    type NestedDiscretionaryModels =
        <Self::NestedDiscretionaryTables as NestedTables>::NestedModels;
    type OptionalNestedDiscretionaryModels =
        <Self::NestedDiscretionaryModels as IntoNestedTupleOption>::IntoOptions;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A bundle of a table's insertable model and its associated builders.
pub struct TableBuilderBundle<T: BundlableTableExt> {
    /// The insertable model for the table.
    insertable_model: T::NewValues,
    /// The mandatory associated builders relative to triangular same-as.
    nested_mandatory_associated_builders: T::OptionalMandatoryNestedBuilders,
    /// The discretionary associated builders relative to triangular same-as.
    nested_discretionary_associated_builders: T::OptionalDiscretionaryNestedBuilders,
}

impl<T> Default for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    T::OptionalMandatoryNestedBuilders: Default,
    T::OptionalDiscretionaryNestedBuilders: Default,
{
    fn default() -> Self {
        Self {
            insertable_model: T::default_new_values(),
            nested_mandatory_associated_builders: Default::default(),
            nested_discretionary_associated_builders: Default::default(),
        }
    }
}

impl<T> HasTable for TableBuilderBundle<T>
where
    T: BundlableTableExt,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, C> MayGetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: TypedColumn,
    T::NewValues: MayGetColumn<C>,
{
    #[inline]
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a C::ColumnType>
    where
        C::Table: 'a,
    {
        self.insertable_model.may_get_column_ref()
    }
}

impl<T, C> ValidateColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: TypedColumn<Table = T>,
    T::NewValues: ValidateColumn<C>,
{
    type Error = <T::NewValues as ValidateColumn<C>>::Error;
    type Borrowed = <T::NewValues as ValidateColumn<C>>::Borrowed;

    #[inline]
    fn validate_column_in_context(&self, value: &Self::Borrowed) -> Result<(), Self::Error> {
        self.insertable_model.validate_column_in_context(value)
    }
}

impl<T, C> SetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: HorizontalSameAsGroupExt<Table = T>,
    Self: SetDiscretionarySameAsNestedColumns<
            C::ValueType,
            C::NestedDiscretionaryHorizontalKeys,
            C::NestedDiscretionaryForeignColumns,
        > + SetMandatorySameAsNestedColumns<
            C::ValueType,
            C::NestedMandatoryHorizontalKeys,
            C::NestedMandatoryForeignColumns,
        >,
    T::NewValues: SetColumn<C> + ValidateColumn<C, Error = std::convert::Infallible>,
{
    #[inline]
    fn set_column(&mut self, value: impl Into<C::ColumnType>) -> &mut Self {
        let value = value.into();
        self.set_discretionary_same_as_nested_columns(&value);
        self.set_mandatory_same_as_nested_columns(&value);
        self.insertable_model.set_column(value);
        self
    }
}

impl<T, C> TrySetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: HorizontalSameAsGroupExt<Table = T, ValueType: Borrow<Self::Borrowed>>,
    Self: TrySetDiscretionarySameAsNestedColumns<
            C::ValueType,
            <T::NewValues as ValidateColumn<C>>::Error,
            C::NestedDiscretionaryHorizontalKeys,
            C::NestedDiscretionaryForeignColumns,
        > + TrySetMandatorySameAsNestedColumns<
            C::ValueType,
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
        if let Some(value_ref) = value.as_optional_ref() {
            self.validate_column_in_context(value_ref.borrow())?;
        }
        self.try_set_discretionary_same_as_nested_columns(&value)?;
        self.try_set_mandatory_same_as_nested_columns(&value)?;
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<Key: MandatorySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>, C>
    TrySetMandatorySameAsColumn<Key, C> for TableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::OptionalMandatoryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
    TableBuilder<C::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType> + Clone,
    ) -> Result<&mut Self, Self::Error> {
        if let Some(builder) = self.nested_mandatory_associated_builders.nested_index_mut() {
            builder.try_set_column(value)?;
        }
        Ok(self)
    }
}

impl<Key: DiscretionarySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>, C>
    TrySetDiscretionarySameAsColumn<Key, C> for TableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::OptionalDiscretionaryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
    TableBuilder<Key::ReferencedTable>: TrySetColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn try_set_discretionary_same_as_column(
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

impl<C, T> SetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: MandatorySameAsIndex,
    C::ReferencedTable: BuildableTable,
    T::OptionalMandatoryNestedBuilders: NestedTupleIndexMut<
            <C as MandatorySameAsIndex>::Idx,
            Element = Option<TableBuilder<C::ReferencedTable>>,
        >,
{
    #[inline]
    fn set_mandatory_builder(&mut self, builder: TableBuilder<C::ReferencedTable>) -> &mut Self {
        *self.nested_mandatory_associated_builders.nested_index_mut() = Some(builder);
        self
    }
}

impl<Key> TrySetMandatoryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTableExt,
    Key: MandatorySameAsIndex,
    Key::ReferencedTable: BuildableTable,
    <Key::Table as BundlableTableExt>::OptionalMandatoryNestedBuilders: NestedTupleIndexMut<
            <Key as MandatorySameAsIndex>::Idx,
            Element = Option<TableBuilder<Key::ReferencedTable>>,
        >,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error> {
        *self.nested_mandatory_associated_builders.nested_index_mut() = Some(builder);
        Ok(self)
    }
}

impl<C, T> SetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: DiscretionarySameAsIndex,
    C::ReferencedTable: BuildableTable,
    T::OptionalDiscretionaryNestedBuilders: NestedTupleIndexMut<
            <C as DiscretionarySameAsIndex>::Idx,
            Element = Option<TableBuilder<C::ReferencedTable>>,
        >,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<C::ReferencedTable>,
    ) -> &mut Self {
        *self
            .nested_discretionary_associated_builders
            .nested_index_mut() = Some(builder);
        self
    }
}

impl<Key> TrySetDiscretionaryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTable,
    Key: DiscretionarySameAsIndex,
    Key::ReferencedTable: BuildableTable,
    <Key::Table as BundlableTableExt>::OptionalDiscretionaryNestedBuilders: NestedTupleIndexMut<
            <Key as DiscretionarySameAsIndex>::Idx,
            Element = Option<TableBuilder<Key::ReferencedTable>>,
        >,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error> {
        *self
            .nested_discretionary_associated_builders
            .nested_index_mut() = Some(builder);
        Ok(self)
    }
}
