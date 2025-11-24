//! Submodule defining the `TableBuilder` struct for building Diesel table
//! insertables.

use diesel::associations::HasTable;
use diesel_additions::{
    Insert, MayGetColumn, MayGetInsertableTableModelColumn, OptionTuple, SetColumn,
    SetInsertableTableModelColumn, TableAddition, Tables, TrySetColumn,
    TrySetInsertableTableModelColumn, TrySetInsertableTableModelHomogeneousColumn, TypedColumn,
};
use diesel_relations::vertical_same_as_group::VerticalSameAsGroup;
use typed_tuple::{TypedLast, TypedTuple};

use crate::{
    BuildableColumn, BuildableColumns, BuildableTables, MayGetBuilder, NestedInsert, SetBuilder,
    TrySetBuilder, buildable_table::BuildableTable,
};

/// A builder for creating insertable models for a Diesel table and its
/// ancestors.
pub struct TableBuilder<T: BuildableTable> {
    /// The insertable models for the table and its ancestors.
    insertable_models: <T::AncestorsWithSelf as Tables>::InsertableModels,
    /// The associated builders relative to triangular same-as.
    associated_builders: <<<T::TriangularSameAsColumns as BuildableColumns>::Tables as BuildableTables>::Builders as OptionTuple>::Output,
}

impl<T> HasTable for TableBuilder<T>
where
    T: BuildableTable,
{
    type Table = T;

    fn table() -> Self::Table {
        T::default()
    }
}

impl<C, T> MayGetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: TypedColumn,
    <T::AncestorsWithSelf as Tables>::InsertableModels: MayGetInsertableTableModelColumn<C>,
{
    fn maybe_get(&self) -> Option<&<C as diesel_additions::TypedColumn>::Type> {
        self.insertable_models.maybe_get()
    }
}

impl<C, T> SetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: VerticalSameAsGroup + TypedColumn,
    <T::AncestorsWithSelf as Tables>::InsertableModels: SetInsertableTableModelColumn<C>,
{
    fn set(&mut self, value: &<C as TypedColumn>::Type) {
        self.insertable_models.set(value);
    }
}

impl<C, T> TrySetColumn<C> for TableBuilder<T>
where
    T: BuildableTable,
    C: VerticalSameAsGroup + TypedColumn,
    <T::AncestorsWithSelf as Tables>::InsertableModels: TrySetInsertableTableModelColumn<C>,
    <T::AncestorsWithSelf as Tables>::InsertableModels:
        TrySetInsertableTableModelHomogeneousColumn<C::VerticalSameAsColumns>,
{
    fn try_set(&mut self, value: &<C as TypedColumn>::Type) -> anyhow::Result<()> {
        <<T::AncestorsWithSelf as Tables>::InsertableModels as TrySetInsertableTableModelColumn<
            C,
        >>::try_set(&mut self.insertable_models, value)?;
        <<T::AncestorsWithSelf as Tables>::InsertableModels as TrySetInsertableTableModelHomogeneousColumn<
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

impl<Conn, T> NestedInsert<Conn> for TableBuilder<T>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    T::InsertableModel: Insert<Conn>,
    <T::AncestorsWithSelf as Tables>::InsertableModels:
        TypedLast<<T as TableAddition>::InsertableModel>,
{
    fn nested_insert(
        self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<Self::Table as TableAddition>::Model> {
        let (inserted_model, others) = self.insertable_models.pop();
        inserted_model.insert(conn)
    }
}
