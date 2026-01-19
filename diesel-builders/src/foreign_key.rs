//! Submodule defining a `ForeignKey` trait for Diesel tables.

use tuplities::prelude::*;
mod iter_foreign_key;
pub use iter_foreign_key::{IterDynForeignKeys, IterForeignKeyExt, IterForeignKeys};

use crate::{
    Descendant, GetColumn, TableExt, TypedColumn, TypedNestedTuple, ValueTyped,
    columns::{NonEmptyNestedProjection, NonEmptyProjection},
};

/// A trait defining a table index for Diesel tables.
pub trait TableIndex: NonEmptyProjection {}
impl<I> TableIndex for I where
    I: NonEmptyProjection + NestTuple<Nested: NestedTableIndexTail<typenum::U0, I>>
{
}

/// A trait defining a UNIQUE table index for Diesel tables.
pub trait UniqueTableIndex: NonEmptyProjection {}
impl<I> UniqueTableIndex for I where
    I: NonEmptyProjection + NestTuple<Nested: NestedUniqueTableIndexTail<typenum::U0, I>>
{
}

/// A trait defining a tail of a table index starting from a given index.
///
/// This trait may not define a full index, but only the tail part starting
/// from a given index.
pub trait NestedTableIndexTail<Idx, FullIndex>: NonEmptyNestedProjection {}

impl<Idx, C1, FullIndex> NestedTableIndexTail<Idx, FullIndex> for (C1,)
where
    C1: IndexedColumn<Idx, FullIndex, Table: TableExt>,
    FullIndex: NonEmptyProjection<Table = C1::Table>,
    <FullIndex as NestTuple>::Nested: NestedTupleIndex<Idx, Element = C1>,
{
}

impl<Idx, CHead, Ctail, FullIndex> NestedTableIndexTail<Idx, FullIndex> for (CHead, Ctail)
where
    (CHead, Ctail): NonEmptyNestedProjection<Table = CHead::Table>
        + FlattenNestedTuple<Flattened: NonEmptyProjection<Table = CHead::Table>>,
    CHead: IndexedColumn<Idx, FullIndex>,
    Ctail: NestedTableIndexTail<typenum::Add1<Idx>, FullIndex>,
    Idx: core::ops::Add<typenum::B1>,
    FullIndex: NonEmptyProjection<Table = CHead::Table>,
    <FullIndex as NestTuple>::Nested: NestedTupleIndex<Idx, Element = CHead>,
{
}

/// A trait defining a tail of a UNIQUE table index starting from a given index.
///
/// This trait may not define a full index, but only the tail part starting
/// from a given index.
pub trait NestedUniqueTableIndexTail<Idx, FullIndex>: NonEmptyNestedProjection {}

impl<Idx, C1, FullIndex> NestedUniqueTableIndexTail<Idx, FullIndex> for (C1,)
where
    C1: UniquelyIndexedColumn<Idx, FullIndex, Table: TableExt>,
    FullIndex: NonEmptyProjection<Table = C1::Table>,
    <FullIndex as NestTuple>::Nested: NestedTupleIndex<Idx, Element = C1>,
{
}

impl<Idx, CHead, Ctail, FullIndex> NestedUniqueTableIndexTail<Idx, FullIndex> for (CHead, Ctail)
where
    (CHead, Ctail): NonEmptyNestedProjection<Table = CHead::Table>
        + FlattenNestedTuple<Flattened: NonEmptyProjection<Table = CHead::Table>>,
    CHead: UniquelyIndexedColumn<Idx, FullIndex>,
    Ctail: NestedUniqueTableIndexTail<typenum::Add1<Idx>, FullIndex>,
    Idx: core::ops::Add<typenum::B1>,
    FullIndex: NonEmptyProjection<Table = CHead::Table>,
    <FullIndex as NestTuple>::Nested: NestedTupleIndex<Idx, Element = CHead>,
{
}

/// A trait for Diesel columns which are part of a `NestedTableIndex`.
pub trait IndexedColumn<
    Idx,
    IndexedColumns: NonEmptyProjection<Table = Self::Table, Nested: NestedTupleIndex<Idx, Element = Self>>,
>: TypedColumn
{
}

/// A trait for Diesel columns which are part of a `NestedUniqueTableIndex`.
pub trait UniquelyIndexedColumn<
    Idx,
    UniquelyIndexedColumns: NonEmptyProjection<Table = Self::Table, Nested: NestedTupleIndex<Idx, Element = Self>>,
>: TypedColumn
{
}
impl<Idx, C, IndexedColumns> IndexedColumn<Idx, IndexedColumns> for C
where
    C: UniquelyIndexedColumn<Idx, IndexedColumns>,
    IndexedColumns:
        UniqueTableIndex<Table = C::Table, Nested: NestedTupleIndex<Idx, Element = Self>>,
{
}

/// A trait defining a non-composited primary key column.
pub trait PrimaryKeyColumn: UniquelyIndexedColumn<typenum::U0, (Self,), Table: TableExt> {}
impl<C> PrimaryKeyColumn for C where
    C: UniquelyIndexedColumn<typenum::U0, (C,), Table: TableExt>
        + diesel::Column<Table: diesel::Table<PrimaryKey = C>>
{
}

/// A trait defining a table with a non-composite primary key.
pub trait HasPrimaryKeyColumn:
    TableExt<
        PrimaryKey: PrimaryKeyColumn<Table = Self>,
        NestedPrimaryKeyColumns = (<Self as diesel::Table>::PrimaryKey,),
        Model: GetColumn<<Self as diesel::Table>::PrimaryKey>,
    >
{
}
impl<T> HasPrimaryKeyColumn for T where
    T: TableExt<
            PrimaryKey: PrimaryKeyColumn<Table = Self>,
            NestedPrimaryKeyColumns = (<Self as diesel::Table>::PrimaryKey,),
            Model: GetColumn<<Self as diesel::Table>::PrimaryKey>,
        >
{
}

/// A trait for Diesel columns collections that are part of a foreign key
/// relationship.
pub trait ForeignKey<ReferencedColumns: TableIndex>: NonEmptyProjection {}
impl<ReferencedColumns, HostColumns> ForeignKey<ReferencedColumns> for HostColumns
where
    ReferencedColumns: TableIndex,
    ReferencedColumns::Nested: NestedTableIndexTail<typenum::U0, ReferencedColumns>,
    HostColumns: NonEmptyProjection<
        Nested: NestedForeignKeyTail<
            typenum::U0,
            ReferencedColumns::Nested,
            ReferencedColumns,
            NestedTupleValueType: NestedTupleFrom<
                <ReferencedColumns::Nested as TypedNestedTuple>::NestedTupleValueType,
            >,
        >,
    >,
{
}

/// A trait for Diesel nested columns collections that are a tail of a foreign
/// key from the beginning up to a given index. For instance, if the index is
/// [`U0`](typenum::U0), then the tail includes all columns; if the index is
/// [`U1`](typenum::U1), then the tail includes all columns except the first
/// one, and so on.
pub trait NestedForeignKeyTail<
    Idx,
    ReferencedColumns: NestedTableIndexTail<Idx, FullReferencedIndex>,
    FullReferencedIndex,
>:
    NonEmptyNestedProjection<
    NestedTupleValueType: NestedTupleFrom<
        <ReferencedColumns as TypedNestedTuple>::NestedTupleValueType,
    >,
>
{
}

impl<F1, H1> NestedForeignKeyTail<typenum::U0, (F1,), (F1,)> for (H1,)
where
    H1: TypedColumn + HostColumn<typenum::U0, (H1,), (F1,), Table: TableExt>,
    F1: TypedColumn<ValueType = <H1 as ValueTyped>::ValueType>
        + IndexedColumn<typenum::U0, (F1,), Table: TableExt>,
{
}

impl<Idx, Fhead, Ftail, Hhead, Htail, FullReferencedIndex>
    NestedForeignKeyTail<Idx, (Fhead, Ftail), FullReferencedIndex> for (Hhead, Htail)
where
    Htail: NonEmptyNestedProjection,
    Idx: core::ops::Add<typenum::B1>,
    Ftail: NestedTableIndexTail<typenum::Add1<Idx>, FullReferencedIndex>,
    (Hhead, Htail): NonEmptyNestedProjection<Table = Hhead::Table>,
    (Fhead, Ftail): NestedTableIndexTail<Idx, FullReferencedIndex>,
    Hhead: TypedColumn,
    Fhead: TypedColumn<ValueType = <Hhead as ValueTyped>::ValueType>,
    Htail::NestedTupleValueType: NestedTupleFrom<<Ftail as TypedNestedTuple>::NestedTupleValueType>,
    <(Hhead, Htail) as TypedNestedTuple>::NestedTupleValueType:
        NestedTupleFrom<<(Fhead, Ftail) as TypedNestedTuple>::NestedTupleValueType>,
{
}

/// A trait for Diesel columns that are part of a foreign key relationship.
///
/// This trait should be implemented for each column in a foreign key tuple.
/// Use the `fk!` macro to implement this trait.
pub trait HostColumn<
    Idx,
    HostColumns: ForeignKey<ReferencedColumns, Nested: NestedTupleIndex<Idx, Element = Self>>,
    ReferencedColumns: TableIndex,
>: TypedColumn
{
}

/// A trait for Diesel columns that define single-column foreign key
/// relationships to tables with a singleton primary key.
pub trait ForeignPrimaryKey: TypedColumn {
    /// The referenced table.
    type ReferencedTable: HasPrimaryKeyColumn<
            PrimaryKey: PrimaryKeyColumn<
                ValueType = <Self as ValueTyped>::ValueType,
                ColumnType = <Self as ValueTyped>::ValueType,
            >,
        > + Descendant;
}

impl<C>
    HostColumn<
        typenum::U0,
        (C,),
        (<<C as ForeignPrimaryKey>::ReferencedTable as diesel::Table>::PrimaryKey,),
    > for C
where
    <<C as ForeignPrimaryKey>::ReferencedTable as diesel::Table>::PrimaryKey: PrimaryKeyColumn,
    C: ForeignPrimaryKey<Table: TableExt>,
{
}
