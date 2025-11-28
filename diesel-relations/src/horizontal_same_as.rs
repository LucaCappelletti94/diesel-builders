//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_additions::{
    Columns, ForeignKey, NonCompositePrimaryKeyTableModels, NonCompositePrimaryKeyTables,
    Projection, SingleColumnForeignKey, SingletonForeignKey, TypedColumn,
    columns::NonEmptyProjection, table_addition::HasPrimaryKey,
};
use diesel_builders_macros::impl_horizontal_same_as_keys;

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsColumn<
    KeyColumn: SingleColumnForeignKey<<<Self as Column>::Table as Table>::PrimaryKey>,
    HostColumn: Column<Table = KeyColumn::Table>,
>: Column<Table: HasPrimaryKey>
{
}

impl<KeyColumn, HostColumn, ForeignColumn> HorizontalSameAsColumn<KeyColumn, HostColumn>
    for ForeignColumn
where
    KeyColumn: SingleColumnForeignKey<<<Self as Column>::Table as Table>::PrimaryKey>,
    HostColumn: TypedColumn<Table = KeyColumn::Table>,
    ForeignColumn: TypedColumn<Table: HasPrimaryKey>,
    (KeyColumn, HostColumn):
        ForeignKey<(<<ForeignColumn as Column>::Table as Table>::PrimaryKey, ForeignColumn)>,
{
}

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsKey: SingletonForeignKey {
    /// The set of host columns in the same table which have
    /// an horizontal same-as relationship defined by this key.
    type HostColumns: NonEmptyProjection<Table = Self::Table>;
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type ForeignColumns: NonEmptyProjection<Table = Self::ReferencedTable>;
}

/// A trait for Diesel columns collections that define horizontal same-as
/// relationships.
#[impl_horizontal_same_as_keys]
pub trait HorizontalSameAsKeys<T: diesel::Table>: Projection<T> {
    /// The set of referenced tables.
    type ReferencedTables: NonCompositePrimaryKeyTables<
            PrimaryKeys: Columns<Types = <Self as Columns>::Types>,
            Models: NonCompositePrimaryKeyTableModels,
        >;
}
