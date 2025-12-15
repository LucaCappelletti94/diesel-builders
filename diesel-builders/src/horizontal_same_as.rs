//! Submodule defining an `HorizontalSameAs` trait for Diesel columns.

use typenum::Unsigned;

use tuplities::prelude::*;

use crate::{
    ForeignPrimaryKey, HasPrimaryKeyColumn, NestedBuildableTables, TableExt, TypedNestedTuple,
    ancestors::DescendantWithSelf,
    columns::{
        ColumnsCollection, NestedColumns, NestedColumnsCollection, NonEmptyNestedProjection,
        NonEmptyProjection,
    },
    tables::NonCompositePrimaryKeyNestedTables,
};

/// A trait for Diesel columns collections that define horizontal same-as
/// relationships.
pub trait HorizontalSameAsColumns<
    Key: HorizontalKey<HostColumns = HostColumns, ForeignColumns = Self>,
    HostColumns: NonEmptyProjection,
>: NonEmptyProjection<Table = Key::ReferencedTable>
{
}

impl<Key, HostColumns, ForeignColumns> HorizontalSameAsColumns<Key, HostColumns> for ForeignColumns
where
    Key: HorizontalKey<HostColumns = HostColumns, ForeignColumns = ForeignColumns>,
    HostColumns: NonEmptyProjection,
    ForeignColumns: NonEmptyProjection<Table = Key::ReferencedTable>,
{
}

/// A trait for Diesel columns that define horizontal same-as relationships.
pub trait HorizontalKey:
    ForeignPrimaryKey<ReferencedTable: DescendantWithSelf, Table: HasPrimaryKeyColumn>
{
    /// The set of host columns in the same table which have
    /// an horizontal same-as relationship defined by this key.
    type HostColumns: NonEmptyProjection<Table = Self::Table, Nested: NonEmptyNestedProjection>;
    /// The set of foreign columns in other tables which have
    /// an horizontal same-as relationship defined by this key.
    type ForeignColumns: HorizontalSameAsColumns<Self, Self::HostColumns, Nested: NonEmptyNestedProjection<
        NestedTupleColumnType:  NestedTupleInto<<<Self::HostColumns as NestTuple>::Nested as TypedNestedTuple>::NestedTupleColumnType> + IntoNestedTupleOption<
            IntoOptions: NestedTupleOptionInto<
                <<<Self::HostColumns as NestTuple>::Nested as TypedNestedTuple>::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
            >,
        >,
    >>;
}

/// Extension trait for `HorizontalKey` to access nested host and foreign
/// columns.
pub trait HorizontalKeyExt: HorizontalKey {
    /// The nested host columns.
    type NestedHostColumns: NonEmptyNestedProjection;
    /// The nested foreign columns.
    type NestedForeignColumns: NonEmptyNestedProjection<
        NestedTupleColumnType: NestedTupleInto<
            <Self::NestedHostColumns as TypedNestedTuple>::NestedTupleColumnType,
        > + IntoNestedTupleOption<
            IntoOptions: NestedTupleOptionInto<
                <<Self::NestedHostColumns as TypedNestedTuple>::NestedTupleColumnType as IntoNestedTupleOption>::IntoOptions,
            >,
        >
    >;
}

impl<K> HorizontalKeyExt for K
where
    K: HorizontalKey,
{
    type NestedHostColumns = <K::HostColumns as NestTuple>::Nested;
    type NestedForeignColumns = <K::ForeignColumns as NestTuple>::Nested;
}

/// Index in a tuple for a mandatory same-as relationship.
pub trait MandatorySameAsIndex:
    HorizontalKeyExt<
    NestedHostColumns: NestedTupleStartsWith<<Self::Table as TableExt>::NestedPrimaryKeyColumns>,
>
{
    /// The index in the n-uple of host columns where the mandatory same-as
    /// relationship is defined.
    type Idx: Unsigned;
}

/// Index in a tuple for a discretionary same-as relationship.
pub trait DiscretionarySameAsIndex: HorizontalKeyExt {
    /// The index in the n-uple of host columns where the discretionary same-as
    /// relationship is defined.
    type Idx: Unsigned;
}

/// A trait for Diesel nested columns collections that define horizontal same-as
/// relationships.
pub trait HorizontalNestedKeys<T>: NestedColumns {
    /// The set of referenced tables.
    type NestedReferencedTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns: NestedColumns<
                NestedTupleColumnType = <Self as TypedNestedTuple>::NestedTupleColumnType,
            >,
        > + NestedBuildableTables<
            NestedOptionalBuilders: NestedTupleOptionWith<
                &'static str,
                SameDepth = <Self as NestedColumns>::NestedColumnNames,
            >,
        >;
    /// Tuple of tuples of host columns associated to each horizontal same-as key.
    type NestedHostColumnsMatrix: NestedColumnsCollection<FlattenedMatrix: ColumnsCollection>;
    /// Tuple of tuples of foreign columns associated to each horizontal same-as key.
    type NestedForeignColumnsMatrix: NestedColumnsCollection<FlattenedMatrix: ColumnsCollection>;
}

impl<T> HorizontalNestedKeys<T> for () {
    type NestedReferencedTables = ();
    type NestedHostColumnsMatrix = ();
    type NestedForeignColumnsMatrix = ();
}

impl<Head, T> HorizontalNestedKeys<T> for (Head,)
where
    T: HasPrimaryKeyColumn,
    Head: HorizontalKeyExt<Table = T>,
    (Head,): NestedColumns,
    (Head::ReferencedTable,): NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns: NestedColumns<
                NestedTupleColumnType = <(Head,) as TypedNestedTuple>::NestedTupleColumnType,
            >,
        > + NestedBuildableTables<
            NestedOptionalBuilders: NestedTupleOptionWith<
                &'static str,
                SameDepth = <Self as NestedColumns>::NestedColumnNames,
            >,
        >,
    (Head::NestedHostColumns,): NestedColumnsCollection<FlattenedMatrix: ColumnsCollection>,
    (Head::NestedForeignColumns,): NestedColumnsCollection<FlattenedMatrix: ColumnsCollection>,
{
    type NestedReferencedTables = (Head::ReferencedTable,);
    type NestedHostColumnsMatrix = (Head::NestedHostColumns,);
    type NestedForeignColumnsMatrix = (Head::NestedForeignColumns,);
}

impl<Head, Tail, T> HorizontalNestedKeys<T> for (Head, Tail)
where
    T: HasPrimaryKeyColumn,
    Head: HorizontalKeyExt<Table = T>,
    Tail: HorizontalNestedKeys<T>,
    (Head, Tail):
        NestedColumns<NestedTupleColumnType = (Head::ColumnType, Tail::NestedTupleColumnType)>,
    (Head::ReferencedTable, Tail::NestedReferencedTables): NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns: NestedColumns<
                NestedTupleColumnType = (Head::ColumnType, Tail::NestedTupleColumnType),
            >,
        > + NestedBuildableTables<
            NestedOptionalBuilders: NestedTupleOptionWith<
                &'static str,
                SameDepth = <Self as NestedColumns>::NestedColumnNames,
            >,
        >,
    (Head::NestedHostColumns, Tail::NestedHostColumnsMatrix):
        NestedColumnsCollection<FlattenedMatrix: ColumnsCollection>,
    (Head::NestedForeignColumns, Tail::NestedForeignColumnsMatrix):
        NestedColumnsCollection<FlattenedMatrix: ColumnsCollection>,
{
    type NestedReferencedTables = (Head::ReferencedTable, Tail::NestedReferencedTables);
    type NestedHostColumnsMatrix = (Head::NestedHostColumns, Tail::NestedHostColumnsMatrix);
    type NestedForeignColumnsMatrix =
        (Head::NestedForeignColumns, Tail::NestedForeignColumnsMatrix);
}
