//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::associations::HasTable;

mod completed_table_builder_bundle;
mod core_traits;
mod serde;
pub use completed_table_builder_bundle::CompletedTableBuilderBundle;
pub(crate) use completed_table_builder_bundle::RecursiveBundleInsert;

use crate::tables::TablesExt;
use crate::{
    BuildableTables, HorizontalSameAsKeys, MayGetColumn, NonCompositePrimaryKeyTables,
    TableAddition, Tables, TrySetColumn, TupleGetColumns, TupleMayGetColumns, TypedColumn,
};
use crate::{InsertableTableModel, Typed};
use tuplities::prelude::*;

/// Trait representing a Diesel table with associated mandatory and
/// discretionary triangular same-as columns.
pub trait BundlableTable: TableAddition {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularSameAsColumns: HorizontalSameAsKeys<Self, ReferencedTables: BuildableTables>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<Self, ReferencedTables: BuildableTables>;
}

/// Extension trait for `BundlableTable`.
pub trait BundlableTableExt:
    BundlableTable<
        MandatoryTriangularSameAsColumns: HorizontalSameAsKeys<
            Self,
            ReferencedTables: BuildableTables<
                Builders = Self::MandatoryBuilders,
                OptionalBuilders = Self::OptionalMandatoryBuilders,
                Models = Self::MandatoryModels,
            >,
        >,
        DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<
            Self,
            ReferencedTables: BuildableTables<
                Builders = Self::DiscretionaryBuilders,
                OptionalBuilders = Self::OptionalDiscretionaryBuilders,
                Models = Self::DiscretionaryModels,
                OptionalModels = Self::OptionalDiscretionaryModels,
            >,
        >,
    >
{
    /// Builders for the mandatory associated tables.
    type MandatoryBuilders: IntoTupleOption<IntoOptions = Self::OptionalMandatoryBuilders>;
    /// Optional builders for the mandatory associated tables.
    type OptionalMandatoryBuilders: TupleOption<Transposed = Self::MandatoryBuilders>;
    /// Builders for the discretionary associated tables.
    type DiscretionaryBuilders: IntoTupleOption<IntoOptions = Self::OptionalDiscretionaryBuilders>;
    /// Optional builders for the discretionary associated tables.
    type OptionalDiscretionaryBuilders: TupleOption<Transposed = Self::DiscretionaryBuilders>;
    /// The mandatory models.
    type MandatoryModels: TupleGetColumns<<<Self::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
        Self,
    >>::ReferencedTables as NonCompositePrimaryKeyTables>::PrimaryKeys>;
    /// The discretionary models.
    type DiscretionaryModels: IntoTupleOption<IntoOptions = Self::OptionalDiscretionaryModels> + TupleGetColumns<<<Self::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        Self,
    >>::ReferencedTables as NonCompositePrimaryKeyTables>::PrimaryKeys>;
    /// The optional discretionary models.
    type OptionalDiscretionaryModels: TupleOption<Transposed = Self::DiscretionaryModels> + TupleMayGetColumns<<<Self::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        Self,
    >>::ReferencedTables as NonCompositePrimaryKeyTables>::PrimaryKeys>;
}
impl<T> BundlableTableExt for T
where
    T: BundlableTable,
{
    type MandatoryBuilders = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as BuildableTables>::Builders;
    type OptionalMandatoryBuilders = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as BuildableTables>::OptionalBuilders;
    type DiscretionaryBuilders = <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as BuildableTables>::Builders;
    type OptionalDiscretionaryBuilders = <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as BuildableTables>::OptionalBuilders;
    type MandatoryModels = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as Tables>::Models;
    type DiscretionaryModels = <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as Tables>::Models;
    type OptionalDiscretionaryModels = <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as TablesExt>::OptionalModels;
}

/// A bundle of a table's insertable model and its associated builders.
pub struct TableBuilderBundle<T: BundlableTableExt> {
    /// The insertable model for the table.
    insertable_model: T::InsertableModel,
    /// The mandatory associated builders relative to triangular same-as.
    mandatory_associated_builders: T::OptionalMandatoryBuilders,
    /// The discretionary associated builders relative to triangular same-as.
    discretionary_associated_builders: T::OptionalDiscretionaryBuilders,
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
    fn try_set_column(&mut self, value: <C as Typed>::Type) -> Result<&mut Self, Self::Error> {
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<C, T> crate::SetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: crate::MandatorySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<C as crate::MandatorySameAsIndex>::Idx, Element=Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
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
            Element = Option<
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
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<C as crate::DiscretionarySameAsIndex>::Idx, Element=Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
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
    <<<Key::Table as BundlableTable>::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<Key::Table>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleIndexMut<<Key as crate::DiscretionarySameAsIndex>::Idx, Element=Option<crate::TableBuilder<<Key as crate::SingletonForeignKey>::ReferencedTable>>>,
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
