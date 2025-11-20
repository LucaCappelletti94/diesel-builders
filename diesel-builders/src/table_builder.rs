//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use diesel_additions::{
    MayGetColumn, OptionTuple, SetColumn, TrySetColumn, TrySetHomogeneousColumns, TypedColumn,
    get_set_columns::SetHomogeneousColumns, tables::InsertableTables,
};
use diesel_relations::vertical_same_as_group::VerticalSameAsGroup;

use crate::{
    BuildableColumn, BuildableColumns, BuildableTables, MayGetBuilder, SetBuilder, TrySetBuilder,
    buildable_table::BuildableTable,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    insertable_models: <T::InsertableTables as InsertableTables>::InsertableModels,
    /// The associated builders relative to triangular same-as.
    associated_builders: <<<T::TriangularSameAsColumns as BuildableColumns>::Tables as BuildableTables>::Builders as OptionTuple>::Output,
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: TypedColumn,
    <T::InsertableTables as InsertableTables>::InsertableModels: MayGetColumn<C>,
{
    fn maybe_get(&self) -> Option<&<C as diesel_additions::TypedColumn>::Type> {
        self.insertable_models.maybe_get()
    }
}

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: VerticalSameAsGroup + TypedColumn,
    <T::InsertableTables as InsertableTables>::InsertableModels: SetColumn<C>,
    <T::InsertableTables as InsertableTables>::InsertableModels:
        SetHomogeneousColumns<C::VerticalSameAsColumns>,
{
    fn set(&mut self, value: &<C as TypedColumn>::Type) {
        <<T::InsertableTables as InsertableTables>::InsertableModels as SetColumn<C>>::set(
            &mut self.insertable_models,
            value,
        );
        <<T::InsertableTables as InsertableTables>::InsertableModels as SetHomogeneousColumns<
            C::VerticalSameAsColumns,
        >>::set(&mut self.insertable_models, value);
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: VerticalSameAsGroup + TypedColumn,
    <T::InsertableTables as InsertableTables>::InsertableModels: TrySetColumn<C>,
    <T::InsertableTables as InsertableTables>::InsertableModels:
        TrySetHomogeneousColumns<C::VerticalSameAsColumns>,
{
    fn try_set(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<()> {
        <<T::InsertableTables as InsertableTables>::InsertableModels as TrySetColumn<C>>::try_set(
            &mut self.insertable_models,
            value,
        )?;
        <<T::InsertableTables as InsertableTables>::InsertableModels as TrySetHomogeneousColumns<
            C::VerticalSameAsColumns,
        >>::try_set(&mut self.insertable_models, value)?;
        Ok(())
    }
}

impl<C, T> TrySetBuilder<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: BuildableColumn,
    Self: MayGetColumn<C>,
    <<<T::TriangularSameAsColumns as BuildableColumns>::Tables as BuildableTables>::Builders as OptionTuple>::Output: SetBuilder<C> + MayGetBuilder<C>,
{
    fn try_set(&mut self, builder: TableBuilder<<C as diesel::Column>::Table>) -> anyhow::Result<()> {
        if self.maybe_get().is_some() {
            anyhow::bail!(
                "Column {} was already set in insertable models for table {}.",
                C::NAME,
                core::any::type_name::<T>(),
            );
        }
        if self.associated_builders.maybe_get().is_some() {
            anyhow::bail!(
                "Associated builder for column {} was already set in table {}.",
                C::NAME,
                core::any::type_name::<T>(),
            );
        }

        self.associated_builders.set(builder);
        Ok(())
    }
}
