//! Extended `Table` trait with additional functionality.

use tuplities::prelude::{FlattenNestedTuple, IntoNestedTupleOption, NestedTupleOptionWith};

use crate::{
    NestedColumns, NonOptionalTypedNestedTuple, TableModel, TypedNestedTuple,
    columns::{NonEmptyNestedProjection, NonEmptyProjection},
};

/// Extended trait for Diesel tables.
pub trait TableExt:
    diesel::Table<AllColumns: NonEmptyProjection<Table = Self>>
    + diesel::associations::HasTable
    + Default
    + Copy
{
    /// Name of the table as a static string.
    const TABLE_NAME: &'static str;
    /// The associated Diesel model type for this table.
    type Model: TableModel<Table = Self>;
    /// The nested columns necessary to execute insert operations for this
    /// table.
    type NewRecord: NonEmptyNestedProjection<
            Table = Self,
            NestedTupleColumnType: IntoNestedTupleOption<IntoOptions = Self::NewValues>,
            Flattened: NonEmptyProjection<Table = Self>,
        >;
    /// The nested types representing a `Self::NewColumns` for this table.
    type NewValues: FlattenNestedTuple
        + NestedTupleOptionWith<
            &'static str,
            Transposed = <Self::NewRecord as TypedNestedTuple>::NestedTupleColumnType,
            SameDepth = <Self::NewRecord as NestedColumns>::NestedColumnNames,
        >;
    /// The nested primary key columns of this table.
    type NestedPrimaryKeyColumns: NonEmptyNestedProjection<Table = Self>
        + NonOptionalTypedNestedTuple;
    /// Error type associated with this table, such as for the validation
    /// of values before insertion.
    type Error;

    /// Returns the default values for the new record.
    #[must_use]
    fn default_new_values() -> Self::NewValues;
}

/// Extended trait for Diesel models associated with a table.
pub trait HasTableExt: diesel::associations::HasTable<Table: TableExt> {}

impl<T> HasTableExt for T where T: diesel::associations::HasTable<Table: TableExt> {}
