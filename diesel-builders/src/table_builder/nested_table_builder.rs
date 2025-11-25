//! Submodule defining the `NestedTableBuilder` struct for building Diesel
//! nested table insertables and its `NestedInsert` implementation.

use diesel::associations::HasTable;
use diesel_additions::{Insert, OptionTuple, TableAddition, Tables};

use crate::{BuildableColumns, BuildableTable, BuildableTables, NestedInsert, TableBuilder};

pub(super) struct NestedTableBuilder<T: TableAddition, TS: Tables, CS: BuildableColumns> {
    /// The insertable models for the table and its ancestors.
    insertable_models: <TS as Tables>::InsertableModels,
    /// The associated builders relative to triangular same-as.
    associated_builders:
        <<<CS as BuildableColumns>::Tables as BuildableTables>::Builders as OptionTuple>::Output,
    /// The marker type for the main table.
    _marker: std::marker::PhantomData<T>,
}

impl<T: BuildableTable> From<TableBuilder<T>>
    for NestedTableBuilder<T, T::AncestorsWithSelf, T::TriangularSameAsColumns>
{
    fn from(builder: TableBuilder<T>) -> Self {
        Self {
            insertable_models: builder.insertable_models,
            associated_builders: builder.associated_builders,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, TS, CS> HasTable for NestedTableBuilder<T, TS, CS>
where
    T: TableAddition,
    TS: Tables,
    CS: BuildableColumns,
{
    type Table = T;

    fn table() -> Self::Table {
        T::default()
    }
}

impl<T, Conn> NestedInsert<Conn> for NestedTableBuilder<T, (T,), ()>
where
    Conn: diesel::connection::LoadConnection,
    T: BuildableTable,
    <T as TableAddition>::InsertableModel: Insert<Conn>,
{
    fn nested_insert(
        self,
        conn: &mut Conn,
    ) -> diesel::QueryResult<<Self::Table as TableAddition>::Model> {
        let (insertable_model,) = self.insertable_models;
        insertable_model.insert(conn)
    }
}
