//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::associations::HasTable;

mod completed_table_builder_bundle;
mod core_traits;
mod serde;
pub use completed_table_builder_bundle::CompletedTableBuilderBundle;
pub(crate) use completed_table_builder_bundle::RecursiveBundleInsert;

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
    type MandatoryTriangularSameAsColumns: HorizontalSameAsKeys<
            Self,
            ReferencedTables: BuildableTables
                + NonCompositePrimaryKeyTables<
                    PrimaryKeys: NestTuple<
                        Nested: Typed<
                            Type = <<Self::MandatoryTriangularSameAsColumns as Typed>::Type as NestTuple>::Nested,
                        >,
                    >,
                >,
        > + NestTuple<
            Nested: Typed<
                Type = <<Self::MandatoryTriangularSameAsColumns as Typed>::Type as NestTuple>::Nested,
            >,
        > + Typed<Type: NestTuple + IntoTupleOption<IntoOptions: NestTuple>>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<
            Self,
            ReferencedTables: BuildableTables
                + NonCompositePrimaryKeyTables<
                    PrimaryKeys: NestTuple<
                        Nested: Typed<
                            Type = <<Self::DiscretionaryTriangularSameAsColumns as Typed>::Type as NestTuple>::Nested,
                        >,
                    >,
                >,
        > + NestTuple<
            Nested: Typed<
                Type = <<Self::DiscretionaryTriangularSameAsColumns as Typed>::Type as NestTuple>::Nested,
            >,
        > + Typed<Type: NestTuple + IntoTupleOption<IntoOptions: NestTuple>>;
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
            > + NonCompositePrimaryKeyTables<
                PrimaryKeys = Self::MandatoryForeignPrimaryKeys,
            >,
        > + NestTuple<
            Nested = Self::NestedMandatoryTriangularSameAsColumns,
        > + Typed<Type = Self::MandatoryForeignPrimaryKeyTypes>,
        DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<
            Self,
            ReferencedTables: BuildableTables<
                Builders = Self::DiscretionaryBuilders,
                OptionalBuilders = Self::OptionalDiscretionaryBuilders,
                Models = Self::DiscretionaryModels,
                OptionalModels = Self::OptionalDiscretionaryModels,
            > + NonCompositePrimaryKeyTables<
                PrimaryKeys = Self::DiscretionaryForeignPrimaryKeys,
            >,
        > + NestTuple<
            Nested = Self::NestedDiscretionaryTriangularSameAsColumns,
        > + Typed<
            Type = Self::DiscretionaryForeignPrimaryKeyTypes,
        >,
    >
{
    /// Nested mandatory triangular same-as columns.
    type NestedMandatoryTriangularSameAsColumns: Typed<
        Type = Self::NestedMandatoryForeignPrimaryKeyTypes,
    >;
    /// Nested discretionary triangular same-as columns.
    type NestedDiscretionaryTriangularSameAsColumns: Typed<
        Type = Self::NestedDiscretionaryForeignPrimaryKeyTypes,
    >;
    /// Mandatory foreign primary keys.
    type MandatoryForeignPrimaryKeys: NestTuple<Nested = Self::NestedMandatoryForeignPrimaryKeys>
        + Typed<Type = Self::MandatoryForeignPrimaryKeyTypes>;
    /// Mandatory foreign primary key types.
    type MandatoryForeignPrimaryKeyTypes: NestTuple<Nested = Self::NestedMandatoryForeignPrimaryKeyTypes>
        + TupleRef
        + IntoTupleOption<IntoOptions: NestTuple>;
    /// Nested mandatory foreign primary keys.
    type NestedMandatoryForeignPrimaryKeys: FlattenNestedTuple<Flattened = Self::MandatoryForeignPrimaryKeys>
        + Typed<Type = Self::NestedMandatoryForeignPrimaryKeyTypes>;
    /// Nested mandatory foreign primary keys types.
    type NestedMandatoryForeignPrimaryKeyTypes: FlattenNestedTuple<
        Flattened = Self::MandatoryForeignPrimaryKeyTypes,
    >;
    /// Optional discretionary foreign primary key types.
    type OptionalDiscretionaryForeignPrimaryKeyTypes: TupleOption<Transposed = Self::DiscretionaryForeignPrimaryKeyTypes>
        + NestTuple<Nested = Self::NestedOptionalDiscretionaryForeignPrimaryKeyTypes>;
    /// Nested optional discretionary foreign primary key types.
    type NestedOptionalDiscretionaryForeignPrimaryKeyTypes: FlattenNestedTuple<
        Flattened = Self::OptionalDiscretionaryForeignPrimaryKeyTypes,
    >;
    /// Discretionary foreign primary keys.
    type DiscretionaryForeignPrimaryKeys: NestTuple<Nested = Self::NestedDiscretionaryForeignPrimaryKeys>
        + Typed<Type = Self::DiscretionaryForeignPrimaryKeyTypes>;
    /// Discretionary foreign primary key types.
    type DiscretionaryForeignPrimaryKeyTypes: NestTuple<Nested = Self::NestedDiscretionaryForeignPrimaryKeyTypes>
        + TupleRef
        + IntoTupleOption<IntoOptions = Self::OptionalDiscretionaryForeignPrimaryKeyTypes>;
    /// Nested discretionary foreign primary keys.
    type NestedDiscretionaryForeignPrimaryKeys: FlattenNestedTuple<Flattened = Self::DiscretionaryForeignPrimaryKeys>
        + Typed<Type = Self::NestedDiscretionaryForeignPrimaryKeyTypes>;
    /// Nested discretionary foreign primary key types.
    type NestedDiscretionaryForeignPrimaryKeyTypes: FlattenNestedTuple<
        Flattened = Self::DiscretionaryForeignPrimaryKeyTypes,
    >;
    /// Builders for the mandatory associated tables.
    type MandatoryBuilders: IntoTupleOption<IntoOptions = Self::OptionalMandatoryBuilders>;
    /// Optional builders for the mandatory associated tables.
    type OptionalMandatoryBuilders: TupleOption<Transposed = Self::MandatoryBuilders>;
    /// Builders for the discretionary associated tables.
    type DiscretionaryBuilders: IntoTupleOption<IntoOptions = Self::OptionalDiscretionaryBuilders>;
    /// Optional builders for the discretionary associated tables.
    type OptionalDiscretionaryBuilders: TupleOption<Transposed = Self::DiscretionaryBuilders>;
    /// The mandatory models.
    type MandatoryModels: NestTuple + IntoTupleOption<IntoOptions: NestTuple>;
    /// The nested mandatory models.
    type NestedMandatoryModels: TupleGetColumns<Self::NestedMandatoryForeignPrimaryKeys>;
    /// The discretionary models.
    type DiscretionaryModels: NestTuple
        + IntoTupleOption<IntoOptions = Self::OptionalDiscretionaryModels>;
    /// The nested discretionary models.
    type NestedDiscretionaryModels;
    /// The optional discretionary models.
    type OptionalDiscretionaryModels: TupleOption<Transposed = Self::DiscretionaryModels>
        + NestTuple<Nested = Self::NestedOptionalDiscretionaryModels>;
    /// The nested optional discretionary models.
    type NestedOptionalDiscretionaryModels: TupleMayGetColumns<
        Self::NestedDiscretionaryForeignPrimaryKeys,
    >;
}

impl<T> BundlableTableExt for T
where
    T: BundlableTable,
{
    type NestedMandatoryTriangularSameAsColumns =
        <T::MandatoryTriangularSameAsColumns as NestTuple>::Nested;
    type NestedDiscretionaryTriangularSameAsColumns =
        <T::DiscretionaryTriangularSameAsColumns as NestTuple>::Nested;
    type MandatoryForeignPrimaryKeys =
        <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
            T,
        >>::ReferencedTables as NonCompositePrimaryKeyTables>::PrimaryKeys;
    type MandatoryForeignPrimaryKeyTypes = <Self::MandatoryForeignPrimaryKeys as Typed>::Type;
    type NestedMandatoryForeignPrimaryKeys =
        <Self::MandatoryForeignPrimaryKeys as NestTuple>::Nested;
    type NestedMandatoryForeignPrimaryKeyTypes =
        <Self::MandatoryForeignPrimaryKeyTypes as NestTuple>::Nested;
    type DiscretionaryForeignPrimaryKeys =
        <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
            T,
        >>::ReferencedTables as NonCompositePrimaryKeyTables>::PrimaryKeys;
    type DiscretionaryForeignPrimaryKeyTypes =
        <Self::DiscretionaryForeignPrimaryKeys as Typed>::Type;
    type NestedDiscretionaryForeignPrimaryKeys =
        <Self::DiscretionaryForeignPrimaryKeys as NestTuple>::Nested;
    type NestedDiscretionaryForeignPrimaryKeyTypes =
        <Self::DiscretionaryForeignPrimaryKeyTypes as NestTuple>::Nested;
    type OptionalDiscretionaryForeignPrimaryKeyTypes =
        <Self::DiscretionaryForeignPrimaryKeyTypes as IntoTupleOption>::IntoOptions;
    type NestedOptionalDiscretionaryForeignPrimaryKeyTypes =
        <Self::OptionalDiscretionaryForeignPrimaryKeyTypes as NestTuple>::Nested;
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
    type MandatoryModels =
        <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<
            T,
        >>::ReferencedTables as Tables>::Models;
    type NestedMandatoryModels = <Self::MandatoryModels as NestTuple>::Nested;
    type DiscretionaryModels = <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<
        T,
    >>::ReferencedTables as Tables>::Models;
    type NestedDiscretionaryModels = <Self::DiscretionaryModels as NestTuple>::Nested;
    type OptionalDiscretionaryModels = <Self::DiscretionaryModels as IntoTupleOption>::IntoOptions;
    type NestedOptionalDiscretionaryModels =
        <Self::OptionalDiscretionaryModels as NestTuple>::Nested;
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
    fn may_get_column_ref<'a>(&'a self) -> Option<&'a C::Type>
    where
        C::Table: 'a,
    {
        self.insertable_model.may_get_column_ref()
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
