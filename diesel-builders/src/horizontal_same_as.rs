//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_builders_macros::impl_horizontal_same_as_keys;
use typenum::Unsigned;

use tuplities::prelude::*;

use crate::{
    Columns, ForeignKey, HasPrimaryKeyColumn, NonCompositePrimaryKeyTables, SingletonForeignKey,
    TableIndex, Typed, TypedColumn, ancestors::DescendantWithSelf, columns::NonEmptyProjection,
};

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsColumn<
    KeyColumn: SingletonForeignKey<ReferencedTable = Self::Table>,
    HostColumn: TypedColumn<Table = KeyColumn::Table, Type = <Self as Typed>::Type>,
>: TypedColumn<Table: HasPrimaryKeyColumn>
{
}

impl<KeyColumn, HostColumn, ForeignColumn> HorizontalSameAsColumn<KeyColumn, HostColumn>
    for ForeignColumn
where
    KeyColumn: SingletonForeignKey,
    HostColumn: TypedColumn<Table = KeyColumn::Table, Type = <Self as Typed>::Type>,
    ForeignColumn: TypedColumn<Table = KeyColumn::ReferencedTable>,
    (
        <<ForeignColumn as Column>::Table as Table>::PrimaryKey,
        ForeignColumn,
    ): TableIndex + PairTuple,
    (KeyColumn, HostColumn): ForeignKey<(
            <<ForeignColumn as Column>::Table as Table>::PrimaryKey,
            ForeignColumn,
        )> + PairTuple,
    (KeyColumn,): ForeignKey<(<<ForeignColumn as Column>::Table as Table>::PrimaryKey,)>,
{
}

/// A trait for Diesel columns collections that define horizontal same-as
/// relationships.
pub trait HorizontalSameAsColumns<
    Key: HorizontalSameAsKey<HostColumns = HostColumns, ForeignColumns = Self>,
    HostColumns: Columns,
>:
    NonEmptyProjection<Table = Key::ReferencedTable, Type = <HostColumns as Typed>::Type>
    + TuplePopFront<
        Front: TypedColumn<
            Type = <<<Key as Column>::Table as Table>::PrimaryKey as Typed>::Type,
            Table = Key::ReferencedTable,
        >,
    >
{
}

impl<Key, HostColumns, ForeignColumns> HorizontalSameAsColumns<Key, HostColumns> for ForeignColumns
where
    Key: HorizontalSameAsKey<HostColumns = HostColumns, ForeignColumns = ForeignColumns>,
    HostColumns: Columns,
    ForeignColumns: NonEmptyProjection<Table = Key::ReferencedTable, Type = <HostColumns as Typed>::Type>
        + TuplePopFront<
            Front: TypedColumn<
                Type = <<<Key as Column>::Table as Table>::PrimaryKey as Typed>::Type,
                Table = Key::ReferencedTable,
            >,
        >,
{
}

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalSameAsKey:
    SingletonForeignKey<ReferencedTable: DescendantWithSelf, Table: HasPrimaryKeyColumn>
{
    /// The set of host columns in the same table which have
    /// an horizontal same-as relationship defined by this key.
    type HostColumns: NonEmptyProjection<Table = Self::Table>
        + TupleLen
        + TuplePopFront<Front = <<Self as Column>::Table as Table>::PrimaryKey>;
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type ForeignColumns: HorizontalSameAsColumns<Self, Self::HostColumns>
        + TupleLen<Len = <Self::HostColumns as TupleLen>::Len>;
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
/// relationships. Limited to 8 columns as complex horizontal relationships with more columns lead to unwieldy queries and potential performance issues.
#[impl_horizontal_same_as_keys]
pub trait HorizontalSameAsKeys<T: crate::TableAddition>: Columns {
    /// The set of referenced tables.
    type ReferencedTables: NonCompositePrimaryKeyTables<PrimaryKeys: Columns<Type = <Self as Typed>::Type>>
        + TupleLen;
    /// Tuple of tuples of host columns associated to each horizontal same-as key.
    type HostColumnsMatrix: TupleLen<Len = <Self::ReferencedTables as TupleLen>::Len>;
    /// Tuple of tuples of foreign columns associated to each horizontal same-as key.
    type ForeignColumnsMatrix: TupleLen<Len = <Self::ReferencedTables as TupleLen>::Len>;
}
