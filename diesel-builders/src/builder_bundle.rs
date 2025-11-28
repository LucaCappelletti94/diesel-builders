//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::associations::HasTable;
use diesel_additions::{
    Columns, DefaultTuple, NonCompositePrimaryKeyTableModels, OptionTuple, RefTuple, TableAddition,
    Tables, TransposeOptionTuple, TryMaySetColumns, TrySetColumns,
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

impl<T> HasTable for CompletedTableBuilderBundle<T>
where
    T: BundlableTable,
{
    type Table = T;

    fn table() -> Self::Table {
        T::default()
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
    T: BundlableTable,
    T::InsertableModel: NestedInsert<Conn> + TrySetColumns<T::MandatoryTriangularSameAsColumns> + TryMaySetColumns<T::DiscretionaryTriangularSameAsColumns>,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders: NestedInsertTuple<Conn, ModelsTuple = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: NestedInsertOptionTuple<Conn, OptionModelsTuple = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output>,
{
    fn nested_insert(mut self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        let mandatory_models: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models = self.mandatory_associated_builders.nested_insert_tuple(conn)?;
        let mandatory_primary_keys: <<T::MandatoryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> = mandatory_models.get_primary_keys();
        self.insertable_model.try_set(mandatory_primary_keys)?;
        let discretionary_models: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as OptionTuple>::Output = self.discretionary_associated_builders.nested_insert_option_tuple(conn)?;
        let discretionary_primary_keys: <<<T::DiscretionaryTriangularSameAsColumns as Columns>::Types as RefTuple>::Output<'_> as OptionTuple>::Output = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as Tables>::Models as NonCompositePrimaryKeyTableModels>::may_get_primary_keys(&discretionary_models);
        self.insertable_model.try_may_set(discretionary_primary_keys)?;
        self.insertable_model.nested_insert(conn)
    }
}

/// Trait for n-tuples of TableBuilderBundles, providing conversion to
/// CompletedTableBuilderBundles.
pub trait BuilderBundles: DefaultTuple {
    /// The tuple of completed builder bundles.
    type CompletedBundles;

    /// Attempt to convert all builder bundles to completed builder bundles.
    fn try_complete(self) -> anyhow::Result<Self::CompletedBundles>;
}

// Generate implementations for all tuple sizes (1-32)
#[diesel_builders_macros::impl_builder_bundles]
mod impls {}
