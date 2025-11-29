//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::associations::HasTable;
use diesel_additions::{
    ClonableTuple, Columns, DebuggableTuple, DefaultTuple, FlatInsert, MayGetColumn,
    NonCompositePrimaryKeyTableModels, OptionTuple, RefTuple, SetColumn, TableAddition, Tables,
    TransposeOptionTuple, TryMaySetColumns, TrySetColumn, TrySetColumns, TypedColumn,
};
use diesel_relations::HorizontalSameAsKeys;

use crate::{
    BuildableTables, NestedInsert,
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
	mandatory_associated_builders: <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
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
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
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

impl<T: BundlableTable> core::fmt::Debug for CompletedTableBuilderBundle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CompletedTableBuilderBundle")
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

impl<T> HasTable for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
{
    type Table = T;

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
    fn may_get_column(&self) -> Option<&C::Type> {
        self.insertable_model.may_get_column()
    }
}

impl<T, C> TrySetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn,
    T::InsertableModel: TrySetColumn<C>,
{
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
    fn set_column(&mut self, value: &C::Type) -> &mut Self {
        self.insertable_model.set_column(value);
        self
    }
}

impl<T, C> SetColumn<C> for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn,
    T::InsertableModel: SetColumn<C>,
{
    fn set_column(&mut self, value: &C::Type) -> &mut Self {
        self.insertable_model.set_column(value);
        self
    }
}

impl<T, C> TrySetColumn<C> for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
    C: TypedColumn,
    T::InsertableModel: TrySetColumn<C>,
{
    fn try_set_column(&mut self, value: &C::Type) -> anyhow::Result<&mut Self> {
        self.insertable_model.try_set_column(value)?;
        Ok(self)
    }
}

impl<C, T> crate::SetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: diesel_relations::MandatorySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedTuple<<C as diesel_relations::MandatorySameAsIndex>::Idx, crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: typed_tuple::prelude::TypedTuple<<C as diesel_relations::MandatorySameAsIndex>::Idx, Option<crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>>,
{
    fn set_mandatory_builder(&mut self, builder: crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>) -> &mut Self {
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
    C: diesel_relations::MandatorySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedTuple<<C as diesel_relations::MandatorySameAsIndex>::Idx, crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>,
    <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: typed_tuple::prelude::TypedTuple<<C as diesel_relations::MandatorySameAsIndex>::Idx, Option<crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>>,
{
    fn try_set_mandatory_builder(&mut self, builder: crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        if self.mandatory_associated_builders.get().is_some() {
            anyhow::bail!(
                "Mandatory associated builder for column {} was already set in table {}.",
                C::NAME,
                core::any::type_name::<T>(),
            );
        }
        self.mandatory_associated_builders.apply(|opt| {
            *opt = Some(builder.clone());
        });
        Ok(self)
    }
}

impl<C, T> crate::SetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTable,
    C: diesel_relations::DiscretionarySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedTuple<<C as diesel_relations::DiscretionarySameAsIndex>::Idx, crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: typed_tuple::prelude::TypedTuple<<C as diesel_relations::DiscretionarySameAsIndex>::Idx, Option<crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>>,
{
    fn set_discretionary_builder(&mut self, builder: crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>) -> &mut Self {
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
    C: diesel_relations::DiscretionarySameAsIndex,
    C::ReferencedTable: crate::BuildableTable,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: typed_tuple::prelude::TypedTuple<<C as diesel_relations::DiscretionarySameAsIndex>::Idx, crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: typed_tuple::prelude::TypedTuple<<C as diesel_relations::DiscretionarySameAsIndex>::Idx, Option<crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>>>,
{
    fn try_set_discretionary_builder(&mut self, builder: crate::TableBuilder<<C as diesel_additions::SingletonForeignKey>::ReferencedTable>) -> anyhow::Result<&mut Self> {
        use typed_tuple::prelude::TypedTuple;
        if self.discretionary_associated_builders.get().is_some() {
            anyhow::bail!(
                "Discretionary associated builder for column {} was already set in table {}.",
                C::NAME,
                core::any::type_name::<T>(),
            );
        }
        self.discretionary_associated_builders.apply(|opt| {
            *opt = Some(builder.clone());
        });
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
            return Err(anyhow::anyhow!("Not all mandatory associated builders have been set"));
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
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: NestedInsertOptionTuple<Conn, OptionModelsTuple = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output>,
{
    fn insert(&self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        let mut cloned = self.clone();
        let mandatory_models: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models = cloned.mandatory_associated_builders.nested_insert_tuple(conn)?;
        let mandatory_primary_keys: <<T::MandatoryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> = mandatory_models.get_primary_keys();
        cloned.insertable_model.try_set_columns(mandatory_primary_keys)?;
        let discretionary_models: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output = cloned.discretionary_associated_builders.nested_insert_option_tuple(conn)?;
        let discretionary_primary_keys: <<<T::DiscretionaryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> as OptionTuple>::Output = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as NonCompositePrimaryKeyTableModels>::may_get_primary_keys(&discretionary_models);
        cloned.insertable_model.try_may_set_columns(discretionary_primary_keys)?;
        Ok(cloned.insertable_model.flat_insert(conn)?)
    }
}

/// Trait for n-tuples of TableBuilderBundles, providing conversion to
/// CompletedTableBuilderBundles.
pub trait BuilderBundles: DefaultTuple + ClonableTuple + DebuggableTuple {
    /// The tuple of completed builder bundles.
    type CompletedBundles;

    /// Attempt to convert all builder bundles to completed builder bundles.
    fn try_complete(self) -> anyhow::Result<Self::CompletedBundles>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_builder_bundles]
mod impls {}
