//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table `InsertableModel` and its mandatory and discretionary associated
//! builders.

use diesel::associations::HasTable;
use diesel_additions::{
    Columns, NonCompositePrimaryKeyTables, OptionTuple, RefTuple, TableAddition, Tables,
    TransposeOptionTuple, TrySetColumns,
};
use diesel_relations::HorizontalSameAsKeys;

use crate::{
    BuildableTables, NestedInsert,
    nested_insert::{NestedInsertOptionTuple, NestedInsertTuple},
};

/// Trait representing a Diesel table with associated mandatory and
/// discretionary triangular same-as columns.
pub trait TableBundle: TableAddition {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularSameAsColumns: HorizontalSameAsKeys<ReferencedTables: BuildableTables>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularSameAsColumns: HorizontalSameAsKeys<
        ReferencedTables: BuildableTables,
    >;
}

/// A bundle of a table's insertable model and its associated builders.
pub struct TableBuilderBundle<T: TableBundle> {
	/// The insertable model for the table.
	insertable_model: T::InsertableModel,
	/// The mandatory associated builders relative to triangular same-as.
	mandatory_associated_builders: <<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
}

impl<T> HasTable for TableBuilderBundle<T>
where
    T: TableBundle,
{
    type Table = T;

    fn table() -> Self::Table {
        T::default()
    }
}

/// The build-ready variant of a table builder bundle.
pub struct CompletedTableBuilderBundle<T: TableBundle> {
	/// The insertable model for the table.
	insertable_model: T::InsertableModel,
	/// The mandatory associated builders relative to triangular same-as.
	mandatory_associated_builders: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders,
	/// The discretionary associated builders relative to triangular same-as.
	discretionary_associated_builders: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output,
}

impl<T> HasTable for CompletedTableBuilderBundle<T>
where
    T: TableBundle,
{
    type Table = T;

    fn table() -> Self::Table {
        T::default()
    }
}

impl<T> TryFrom<TableBuilderBundle<T>> for CompletedTableBuilderBundle<T>
where
    T: TableBundle,
{
    type Error = diesel::result::Error;

    fn try_from(
        value: TableBuilderBundle<T>,
    ) -> Result<CompletedTableBuilderBundle<T>, Self::Error> {
        let Some(mandatory_associated_builders) =
            value.mandatory_associated_builders.transpose_option()
        else {
            return Err(diesel::result::Error::NotFound);
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
    T: TableBundle,
    T::InsertableModel: NestedInsert<Conn> + TrySetColumns<T::MandatoryTriangularSameAsColumns>,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders: NestedInsertTuple<Conn, ModelsTuple = <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as Tables>::Models>,
    <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as crate::BuildableTables>::Builders as diesel_additions::OptionTuple>::Output: NestedInsertOptionTuple<Conn, OptionModelsTuple = <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as Tables>::Models as OptionTuple>::Output>,
{
    fn nested_insert(self, conn: &mut Conn) -> anyhow::Result<<T as TableAddition>::Model> {
        let mandatory_models: <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as Tables>::Models = self.mandatory_associated_builders.nested_insert_tuple(conn)?;
        let mandatory_primary_keys: <<<<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as NonCompositePrimaryKeyTables>::PrimaryKeys as Columns>::Types as RefTuple>::Output<'_> = todo!();
        let discretionary_models: <<<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys>::ReferencedTables as Tables>::Models as OptionTuple>::Output = self.discretionary_associated_builders.nested_insert_option_tuple(conn)?;
        self.insertable_model.try_set(mandatory_primary_keys)?;
        self.insertable_model.nested_insert(conn)
    }
}
