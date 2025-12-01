//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_builders_macros::impl_horizontal_same_as_keys;
use typed_tuple::prelude::{NthIndex, TypedFirst, U0, Unsigned};

use crate::{
    Columns, ForeignKey, NonCompositePrimaryKeyTableModels, NonCompositePrimaryKeyTables,
    Projection, SingletonForeignKey, TableIndex, TypedColumn, ancestors::DescendantWithSelf,
    columns::NonEmptyProjection, table_addition::HasPrimaryKey,
};

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsColumn<
    KeyColumn: SingletonForeignKey<ReferencedTable = Self::Table>,
    HostColumn: TypedColumn<Table = KeyColumn::Table, Type = <Self as TypedColumn>::Type>,
>: TypedColumn<Table: HasPrimaryKey>
{
}

impl<KeyColumn, HostColumn, ForeignColumn> HorizontalSameAsColumn<KeyColumn, HostColumn>
    for ForeignColumn
where
    KeyColumn: SingletonForeignKey<ReferencedTable = ForeignColumn::Table>,
    HostColumn: TypedColumn<Table = KeyColumn::Table, Type = <Self as TypedColumn>::Type>,
    ForeignColumn: TypedColumn<Table: HasPrimaryKey>,
    (
        <<ForeignColumn as Column>::Table as Table>::PrimaryKey,
        ForeignColumn,
    ): TableIndex,
    (KeyColumn, HostColumn): ForeignKey<(
        <<ForeignColumn as Column>::Table as Table>::PrimaryKey,
        ForeignColumn,
    )>,
    (KeyColumn,): ForeignKey<(<<ForeignColumn as Column>::Table as Table>::PrimaryKey,)>,
{
}

/// A trait for Diesel columns collections that define horizontal same-as
/// relationships.
#[diesel_builders_macros::impl_horizontal_same_as_columns]
pub trait HorizontalSameAsColumns<
    Key: HorizontalSameAsKey<HostColumns = HostColumns, ForeignColumns = Self>,
    HostColumns: Columns,
>:
    NonEmptyProjection<Table = Key::ReferencedTable, Types = <HostColumns as Columns>::Types>
    + NthIndex<
        U0,
        NthType: TypedColumn<
            Type = <<<Key as Column>::Table as Table>::PrimaryKey as TypedColumn>::Type,
            Table = Key::ReferencedTable,
        >,
    >
{
}

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsKey:
    SingletonForeignKey<ReferencedTable: DescendantWithSelf, Table: HasPrimaryKey>
{
    /// The set of host columns in the same table which have
    /// an horizontal same-as relationship defined by this key.
    type HostColumns: NonEmptyProjection<Table = Self::Table>
        + TypedFirst<<<Self as Column>::Table as Table>::PrimaryKey>;
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type ForeignColumns: HorizontalSameAsColumns<Self, Self::HostColumns>;
}

/// Index in a tuple for a mandatory same-as relationship.
pub trait MandatorySameAsIndex: HorizontalSameAsKey {
    /// The index in the n-uple of host columns where the mandatory same-as
    /// relationship is defined.
    type Idx: Unsigned;
}

/// Index in a tuple for a discretionary same-as relationship.
pub trait DiscretionarySameAsIndex: HorizontalSameAsKey {
    /// The index in the n-uple of host columns where the discretionary same-as
    /// relationship is defined.
    type Idx: Unsigned;
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
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type FirstForeignColumns: Columns;
}
