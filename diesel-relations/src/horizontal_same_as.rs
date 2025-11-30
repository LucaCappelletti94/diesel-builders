//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_additions::{
    Columns, ForeignKey, NonCompositePrimaryKeyTableModels, NonCompositePrimaryKeyTables,
    Projection, SingleColumnForeignKey, SingletonForeignKey, TypedColumn,
    columns::NonEmptyProjection, table_addition::HasPrimaryKey,
};
use diesel_builders_macros::impl_horizontal_same_as_keys;
use typed_tuple::prelude::{NthIndex, TupleIndex, TupleIndex0, TypedFirst};

use crate::ancestors::DescendantWithSelf;

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
pub trait HorizontalSameAsKey:
    SingletonForeignKey<ReferencedTable: DescendantWithSelf, Table: HasPrimaryKey>
{
    /// The set of host columns in the same table which have
    /// an horizontal same-as relationship defined by this key.
    type HostColumns: NonEmptyProjection<Table = Self::Table>
        + TypedFirst<<<Self as Column>::Table as Table>::PrimaryKey>;
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type ForeignColumns: NonEmptyProjection<
            Table = Self::ReferencedTable,
            Types = <Self::HostColumns as Columns>::Types,
        > + NthIndex<
            TupleIndex0,
            NthType: TypedColumn<
                Type = <<<Self as Column>::Table as Table>::PrimaryKey as TypedColumn>::Type,
                Table = Self::ReferencedTable,
            >,
        >;
}

/// Index in a tuple for a mandatory same-as relationship.
pub trait MandatorySameAsIndex: HorizontalSameAsKey {
    /// The index in the n-uple of host columns where the mandatory same-as
    /// relationship is defined.
    type Idx: TupleIndex;
}

/// Index in a tuple for a discretionary same-as relationship.
pub trait DiscretionarySameAsIndex: HorizontalSameAsKey {
    /// The index in the n-uple of host columns where the discretionary same-as
    /// relationship is defined.
    type Idx: TupleIndex;
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
