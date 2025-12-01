//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::{Column, associations::HasTable};
use typed_tuple::prelude::TypedIndex;

use crate::{
    BuildableTable, BuildableTables, ClonableTuple, Columns, DebuggableTuple, DefaultTuple,
    DiscretionarySameAsIndex, FlatInsert, HorizontalSameAsGroup, HorizontalSameAsKeys,
    MandatorySameAsIndex, MayGetColumn, MaySetColumn, NestedInsert,
    NonCompositePrimaryKeyTableModels, OptionTuple, RefTuple, SetColumn, TableAddition,
    TableBuilder, Tables, TransposeOptionTuple, TryMaySetColumns,
    TryMaySetDiscretionarySameAsColumn, TryMaySetDiscretionarySameAsColumns, TrySetColumn,
    TrySetColumns, TrySetMandatorySameAsColumn, TrySetMandatorySameAsColumns, TypedColumn,
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

impl<T: BundlableTable> Clone for TableBuilderBundle<T> {
    fn clone(&self) -> Self {
        Self {
            insertable_model: self.insertable_model.clone(),
            mandatory_associated_builders: self.mandatory_associated_builders.clone_tuple(),
            discretionary_associated_builders: self.discretionary_associated_builders.clone_tuple(),
        }
    }
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

impl<T: BundlableTable> Clone for CompletedTableBuilderBundle<T> {
    fn clone(&self) -> Self {
        Self {
            insertable_model: self.insertable_model.clone(),
            mandatory_associated_builders: self.mandatory_associated_builders.clone_tuple(),
            discretionary_associated_builders: self.discretionary_associated_builders.clone_tuple(),
        }
    }
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

impl<T, C> MaySetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn,
    T::InsertableModel: MaySetColumn<C>,
{
    #[inline]
    fn may_set_column(&mut self, value: Option<&C::Type>) -> &mut Self {
        self.insertable_model.may_set_column(value);
        self
    }
}

impl<T, C> TrySetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn<Table = T>,
    T::InsertableModel: TrySetColumn<C>,
{
    #[inline]
    fn try_set_column(&mut self, value: &C::Type) -> anyhow::Result<&mut Self> {
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<T, C> SetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn,
    T::InsertableModel: SetColumn<C>,
{
    #[inline]
    fn set_column(&mut self, value: &C::Type) -> &mut Self {
        self.insertable_model.set_column(value);
        self
    }
}

impl<T, C> TrySetColumn<C> for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn<Table=<Self as HasTable>::Table> + HorizontalSameAsGroup,
    Self: TryMaySetDiscretionarySameAsColumns<
        C::Type,
        C::DiscretionaryHorizontalSameAsKeys,
        <C::DiscretionaryHorizontalSameAsKeys as HorizontalSameAsKeys<C::Table>>::FirstForeignColumns,
    >,
    Self: TrySetMandatorySameAsColumns<
        C::Type,
        C::MandatoryHorizontalSameAsKeys,
        <C::MandatoryHorizontalSameAsKeys as HorizontalSameAsKeys<C::Table>>::FirstForeignColumns,
    >,
    T::InsertableModel: TrySetColumn<C>,
{
    #[inline]
    fn try_set_column(&mut self, value: &C::Type) -> anyhow::Result<&mut Self> {
        self.try_may_set_discretionary_same_as_columns(value)?;
        self.try_set_mandatory_same_as_columns(value)?;
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
        use typed_tuple::prelude::TypedTuple;
        self.mandatory_associated_builders.apply(|opt| {
            *opt = Some(builder.clone());
        });
        self
    }
}

impl<C, T> crate::TrySetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: crate::MandatorySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedIndex<<C as crate::MandatorySameAsIndex>::Idx, crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: typed_tuple::prelude::TypedIndex<<C as crate::MandatorySameAsIndex>::Idx, Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn try_set_mandatory_builder(&mut self, builder: crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        self.mandatory_associated_builders.apply(|opt| {
            *opt = Some(builder.clone());
        });
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
        use typed_tuple::prelude::TypedTuple;
        self.discretionary_associated_builders.apply(|opt| {
            *opt = Some(builder.clone());
        });
        self
    }
}

impl<C, T> crate::TrySetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: crate::DiscretionarySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedIndex<<C as crate::DiscretionarySameAsIndex>::Idx, crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: typed_tuple::prelude::TypedIndex<<C as crate::DiscretionarySameAsIndex>::Idx, Option<crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>>>,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: crate::TableBuilder<<C as crate::SingletonForeignKey>::ReferencedTable>,
    ) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        self.discretionary_associated_builders.apply(|opt| {
            *opt = Some(builder.clone());
        });
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
    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: &<C as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        self.mandatory_associated_builders.map_mut(|builder: &mut TableBuilder<<C as Column>::Table>| {
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
    #[inline]
    fn try_may_set_discretionary_same_as_column(
        &mut self,
        value: &<C as TypedColumn>::Type,
    ) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        self.discretionary_associated_builders.map_mut(|opt_builder: &mut Option<TableBuilder<<C as Column>::Table>>| {
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
    type Error = anyhow::Error;

    fn try_from(
        value: TableBuilderBundle<T>,
    ) -> Result<CompletedTableBuilderBundle<T>, Self::Error> {
        let Some(mandatory_associated_builders) =
            value.mandatory_associated_builders.transpose_option()
        else {
            return Err(anyhow::anyhow!(
                "Not all mandatory associated builders have been set"
            ));
        };
        Ok(CompletedTableBuilderBundle {
            insertable_model: value.insertable_model,
            mandatory_associated_builders,
            discretionary_associated_builders: value.discretionary_associated_builders,
        })
    }
}

impl<T, Conn> NestedInsert<Conn> for CompletedTableBuilderBundle<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BundlableTable,
    T::InsertableModel: FlatInsert<Conn> + TrySetColumns<T::MandatoryTriangularSameAsColumns> + TryMaySetColumns<T::DiscretionaryTriangularSameAsColumns>,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: NestedInsertTuple<Conn, ModelsTuple = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as crate::OptionTuple>::Output: NestedInsertOptionTuple<Conn, OptionModelsTuple = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output>,
{
    fn insert(mut self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        let mandatory_models: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models = self.mandatory_associated_builders.nested_insert_tuple(conn)?;
        let mandatory_primary_keys: <<T::MandatoryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> = mandatory_models.get_primary_keys();
        self.insertable_model.try_set_columns(mandatory_primary_keys)?;
        let discretionary_models: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output = self.discretionary_associated_builders.nested_insert_option_tuple(conn)?;
        let discretionary_primary_keys: <<<T::DiscretionaryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> as OptionTuple>::Output = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as NonCompositePrimaryKeyTableModels>::may_get_primary_keys(&discretionary_models);
        self.insertable_model.try_may_set_columns(discretionary_primary_keys)?;
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
    fn try_complete(self) -> anyhow::Result<Self::CompletedBundles>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_builder_bundles]
mod impls {}
