//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::associations::HasTable;

mod completed_table_builder_bundle;
mod serde;
pub use completed_table_builder_bundle::CompletedTableBuilderBundle;
pub(crate) use completed_table_builder_bundle::RecursiveBundleInsert;

use crate::columns::NestedColumns;
use crate::tables::NonCompositePrimaryKeyNestedTables;
use crate::{
    BuildableTable, DiscretionarySameAsIndex, HasNestedTables, HorizontalKeys,
    HorizontalSameAsNestedKeys, InsertableTableModel, MandatorySameAsIndex, NestedBuildableTables,
    NestedTableModels, NestedTables, SetDiscretionaryBuilder, SetMandatoryBuilder, TableBuilder,
    TrySetDiscretionaryBuilder, TrySetMandatoryBuilder, TupleMayGetNestedColumns, Typed,
    TypedNestedTuple,
};
use crate::{MayGetColumn, TableExt, TrySetColumn, TupleGetNestedColumns, TypedColumn};
use tuplities::prelude::*;

/// Trait representing a Diesel table with associated mandatory and
/// discretionary triangular same-as columns.
pub trait BundlableTable: Sized {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularColumns: HorizontalKeys<Self>
        + NestTuple<Nested: HorizontalSameAsNestedKeys<Self>>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularColumns: HorizontalKeys<Self>
        + NestTuple<Nested: HorizontalSameAsNestedKeys<Self>>;
}

/// Extension trait for `BundlableTable`.
pub trait BundlableTableExt: BundlableTable {
    /// Nested mandatory triangular same-as columns.
    type NestedMandatoryTriangularColumns: NestedColumns<
        NestedTupleType = Self::NestedMandatoryPrimaryKeyTypes,
    >;
    /// Nested mandatory tables.
    type NestedMandatoryTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns = Self::NestedMandatoryPrimaryKeys,
            NestedModels = Self::NestedMandatoryModels,
        > + NestedBuildableTables;
    /// Nested discretionary triangular same-as columns.
    type NestedDiscretionaryTriangularColumns: NestedColumns<
        NestedTupleType = Self::NestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested discretionary tables.
    type NestedDiscretionaryTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns = Self::NestedDiscretionaryPrimaryKeys,
            NestedModels = Self::NestedDiscretionaryModels,
            OptionalNestedModels = Self::OptionalNestedDiscretionaryModels,
        > + NestedBuildableTables;
    /// Nested mandatory foreign primary keys.
    type NestedMandatoryPrimaryKeys: NestedColumns<
        NestedTupleType = Self::NestedMandatoryPrimaryKeyTypes,
    >;
    /// Nested mandatory foreign primary keys types.
    type NestedMandatoryPrimaryKeyTypes;
    /// Nested discretionary foreign primary keys.
    type NestedDiscretionaryPrimaryKeys: NestedColumns<
        NestedTupleType = Self::NestedDiscretionaryPrimaryKeyTypes,
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
    type OptionalMandatoryNestedBuilders: NestedTupleOption<Transposed = Self::MandatoryNestedBuilders>
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
    T: BundlableTable,
{
    type NestedMandatoryTriangularColumns = <T::MandatoryTriangularColumns as NestTuple>::Nested;
    type NestedMandatoryTables =
        <Self::NestedMandatoryTriangularColumns as HorizontalSameAsNestedKeys<
            T,
        >>::NestedReferencedTables;
    type NestedDiscretionaryTriangularColumns =
        <T::DiscretionaryTriangularColumns as NestTuple>::Nested;
    type NestedDiscretionaryTables =
        <Self::NestedDiscretionaryTriangularColumns as HorizontalSameAsNestedKeys<
            T,
        >>::NestedReferencedTables;
    type NestedMandatoryPrimaryKeys =
        <Self::NestedMandatoryTables as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns;
    type NestedMandatoryPrimaryKeyTypes =
        <Self::NestedMandatoryPrimaryKeys as TypedNestedTuple>::NestedTupleType;
    type NestedDiscretionaryPrimaryKeys =
        <Self::NestedDiscretionaryTables as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns;
    type NestedDiscretionaryPrimaryKeyTypes =
        <Self::NestedDiscretionaryPrimaryKeys as TypedNestedTuple>::NestedTupleType;
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A bundle of a table's insertable model and its associated builders.
pub struct TableBuilderBundle<T: BundlableTableExt + TableExt> {
    /// The insertable model for the table.
    insertable_model: T::InsertableModel,
    /// The mandatory associated builders relative to triangular same-as.
    nested_mandatory_associated_builders: T::OptionalMandatoryNestedBuilders,
    /// The discretionary associated builders relative to triangular same-as.
    nested_discretionary_associated_builders: T::OptionalDiscretionaryNestedBuilders,
}

impl<T> HasTable for TableBuilderBundle<T>
where
    T: BundlableTable + TableExt,
{
    type Table = T;

    #[inline]
    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, C> MayGetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable + TableExt,
    C: TypedColumn,
    T::InsertableModel: MayGetColumn<C>,
{
    #[inline]
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a C::Type>
    where
        C::Table: 'a,
    {
        self.insertable_model.may_get_column_ref()
    }
}

impl<T, C> TrySetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable + TableExt,
    C: TypedColumn<Table = T>,
    T::InsertableModel: TrySetColumn<C>,
{
    type Error = <T::InsertableModel as TrySetColumn<C>>::Error;

    #[inline]
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<C, T> SetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt + TableExt,
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
    ) -> Result<
        &mut Self,
        <<<Self as HasTable>::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    > {
        *self.nested_mandatory_associated_builders.nested_index_mut() = Some(builder);
        Ok(self)
    }
}

impl<C, T> SetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt + TableExt,
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
    ) -> Result<
        &mut Self,
        <<<Self as HasTable>::Table as TableExt>::InsertableModel as InsertableTableModel>::Error,
    > {
        *self
            .nested_discretionary_associated_builders
            .nested_index_mut() = Some(builder);
        Ok(self)
    }
}
