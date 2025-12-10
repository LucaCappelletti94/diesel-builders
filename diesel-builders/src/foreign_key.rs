//! Submodule defining a `ForeignKey` trait for Diesel tables.

use tuplities::prelude::*;

use crate::{
    GetColumn, TableExt, Typed, TypedColumn, TypedNestedTuple, TypedTuple,
    columns::{NonEmptyNestedProjection, NonEmptyProjection},
};

/// A trait defining a table index for Diesel tables.
pub trait TableIndex: NonEmptyProjection {}
impl<I> TableIndex for I where
    I: NonEmptyProjection + NestTuple<Nested: NestedTableIndexTail<typenum::U0, I>>
{
}

/// A trait defining a tail of a table index starting from a given index.
///
/// This trait may not define a full index, but only the tail part starting
/// from a given index.
pub trait NestedTableIndexTail<Idx, FullIndex>: NonEmptyNestedProjection {}

impl<Idx, C1, FullIndex> NestedTableIndexTail<Idx, FullIndex> for (C1,)
where
    C1: IndexedColumn<Idx, FullIndex>,
    FullIndex: NonEmptyProjection<Table = C1::Table>,
    <FullIndex as NestTuple>::Nested: NestedTupleIndex<Idx, Element = C1>,
{
}

impl<Idx, Chead, Ctail, FullIndex> NestedTableIndexTail<Idx, FullIndex> for (Chead, Ctail)
where
    (Chead, Ctail): NonEmptyNestedProjection<Table = Chead::Table>
        + FlattenNestedTuple<Flattened: NonEmptyProjection<Table = Chead::Table>>,
    Chead: IndexedColumn<Idx, FullIndex>,
    Ctail: NestedTableIndexTail<typenum::Add1<Idx>, FullIndex>,
    Idx: core::ops::Add<typenum::B1>,
    FullIndex: NonEmptyProjection<Table = Chead::Table>,
    <FullIndex as NestTuple>::Nested: NestedTupleIndex<Idx, Element = Chead>,
{
}

/// A trait for Diesel columns which are part of a `NestedTableIndex`.
pub trait IndexedColumn<Idx, IndexedColumns: NonEmptyProjection<Table = Self::Table>>:
    TypedColumn
where
    <IndexedColumns as NestTuple>::Nested: NestedTupleIndex<Idx, Element = Self>,
{
}

/// A trait defining a non-composited primary key column.
pub trait PrimaryKeyColumn: IndexedColumn<typenum::U0, (Self,)> {}
impl<C> PrimaryKeyColumn for C where
    C: IndexedColumn<typenum::U0, (C,)> + diesel::Column<Table: diesel::Table<PrimaryKey = C>>
{
}

/// A trait defining a table with a non-composite primary key.
pub trait HasPrimaryKeyColumn:
    TableExt<
        PrimaryKey: PrimaryKeyColumn<Table = Self>,
        PrimaryKeyColumns: TuplePopFront<Front = <Self as diesel::Table>::PrimaryKey>,
        Model: GetColumn<<Self as diesel::Table>::PrimaryKey>,
    >
{
}
impl<T> HasPrimaryKeyColumn for T where
    T: TableExt<
            PrimaryKey: PrimaryKeyColumn<Table = Self>,
            PrimaryKeyColumns: TuplePopFront<Front = <Self as diesel::Table>::PrimaryKey>,
            Model: GetColumn<<Self as diesel::Table>::PrimaryKey>,
        >
{
}

/// A trait for Diesel columns collections that are part of a foreign key
/// relationship.
pub trait ForeignKey<ReferencedColumns: TableIndex>: NonEmptyProjection {}
impl<ReferencedColumns, HostColumns> ForeignKey<ReferencedColumns> for HostColumns
where
    ReferencedColumns: TableIndex<TupleType = <HostColumns as TypedTuple>::TupleType>,
    <ReferencedColumns as NestTuple>::Nested: NestedTableIndexTail<
        typenum::U0,
        ReferencedColumns,
        NestedTupleType = <<HostColumns as NestTuple>::Nested as TypedNestedTuple>::NestedTupleType,
    >,
    HostColumns: NonEmptyProjection<
        Nested: NestedForeignKeyTail<typenum::U0, ReferencedColumns::Nested, ReferencedColumns>,
    >,
{
}

/// A trait for Diesel nested columns collections that are a tail of a foreign key from
/// the beginning up to a given index. For instance, if the index is [`U0`](typenum::U0), then the
/// tail includes all columns; if the index is [`U1`](typenum::U1), then the tail includes all columns
/// except the first one, and so on.
pub trait NestedForeignKeyTail<
    Idx,
    ReferencedColumns: NestedTableIndexTail<
            Idx,
            FullReferencedIndex,
            NestedTupleType = <Self as TypedNestedTuple>::NestedTupleType,
        >,
    FullReferencedIndex,
>: NonEmptyNestedProjection
{
}

impl<F1, H1> NestedForeignKeyTail<typenum::U0, (F1,), (F1,)> for (H1,)
where
    H1: TypedColumn,
    F1: TypedColumn<Type = <H1 as Typed>::Type>,
    F1: IndexedColumn<typenum::U0, (F1,)>,
    H1: HostColumn<typenum::U0, (H1,), (F1,)>,
{
}

impl<Idx, Fhead, Ftail, Hhead, Htail, FullReferencedIndex>
    NestedForeignKeyTail<Idx, (Fhead, Ftail), FullReferencedIndex> for (Hhead, Htail)
where
    Htail: NonEmptyNestedProjection,
    Idx: core::ops::Add<typenum::B1>,
    Ftail: NestedTableIndexTail<
            typenum::Add1<Idx>,
            FullReferencedIndex,
            NestedTupleType = <Htail as TypedNestedTuple>::NestedTupleType,
        >,
    (Hhead, Htail): NonEmptyNestedProjection<Table = Hhead::Table>,
    (Fhead, Ftail): NestedTableIndexTail<
            Idx,
            FullReferencedIndex,
            NestedTupleType = <(Hhead, Htail) as TypedNestedTuple>::NestedTupleType,
        >,
    Hhead: TypedColumn,
    Fhead: TypedColumn<Type = <Hhead as Typed>::Type>,
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
pub trait SingletonForeignKey: TypedColumn {
    /// The referenced table.
    type ReferencedTable: HasPrimaryKeyColumn<
            PrimaryKey: PrimaryKeyColumn<Type = <Self as Typed>::Type>,
            PrimaryKeyColumns: TuplePopFront<Front: PrimaryKeyColumn<Type = <Self as Typed>::Type>>,
        > + diesel::query_source::TableNotEqual<Self::Table>;
}

impl<C>
    HostColumn<
        typenum::U0,
        (C,),
        (<<C as SingletonForeignKey>::ReferencedTable as diesel::Table>::PrimaryKey,),
    > for C
where
    <<C as SingletonForeignKey>::ReferencedTable as diesel::Table>::PrimaryKey: PrimaryKeyColumn,
    C: SingletonForeignKey,
{
}
