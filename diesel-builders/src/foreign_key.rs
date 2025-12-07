//! Submodule defining a `ForeignKey` trait for Diesel tables.

use typenum::{U0, Unsigned};

use tuplities::prelude::*;

use crate::{GetColumn, TableAddition, Typed, TypedColumn, columns::NonEmptyProjection};

/// A trait for Diesel tables that define indices which
/// can be used by foreign keys. Limited to 8 columns as composite foreign keys with more than 8 columns are extremely rare and indicate poor design.
#[diesel_builders_macros::impl_table_index]
pub trait TableIndex: NonEmptyProjection + TupleLen {}

/// A trait for Diesel columns which are part of a `TableIndex`.
pub trait IndexedColumn<
    Idx: Unsigned,
    IndexedColumns: TableIndex<Table = Self::Table> + TupleIndex<Idx, Element = Self>,
>: TypedColumn
{
}

/// A trait defining a non-composited primary key column.
pub trait PrimaryKeyColumn: IndexedColumn<U0, (Self,)> {}
impl<C> PrimaryKeyColumn for C where C: IndexedColumn<U0, (C,)> {}

/// A trait defining a table with a non-composite primary key.
pub trait HasPrimaryKeyColumn:
    TableAddition<
        PrimaryKey: PrimaryKeyColumn<Table = Self>,
        PrimaryKeyColumns: TuplePopFront<Front = <Self as diesel::Table>::PrimaryKey>,
        Model: GetColumn<<Self as diesel::Table>::PrimaryKey>,
    >
{
}
impl<T> HasPrimaryKeyColumn for T where
    T: TableAddition<
            PrimaryKey: PrimaryKeyColumn<Table = Self>,
            PrimaryKeyColumns: TuplePopFront<Front = <Self as diesel::Table>::PrimaryKey>,
            Model: GetColumn<<Self as diesel::Table>::PrimaryKey>,
        >
{
}

/// A trait for Diesel tables that define foreign key relationships.
/// A trait for Diesel tables that define foreign key relationships. Limited to 8 columns as foreign keys with more than 8 columns are impractical and suggest design issues.
#[diesel_builders_macros::impl_foreign_key]
pub trait ForeignKey<ReferencedColumns: TableIndex>: NonEmptyProjection {}

/// A trait for Diesel columns that are part of a foreign key relationship.
///
/// This trait should be implemented for each column in a foreign key tuple.
/// Use the `fk!` macro to implement this trait.
pub trait HostColumn<
    Idx: Unsigned,
    HostColumns: ForeignKey<ReferencedColumns> + TupleIndex<Idx, Element = Self>,
    ReferencedColumns: TableIndex + TupleIndex<Idx, Element: TypedColumn<Type = <Self as Typed>::Type>>,
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
        U0,
        (C,),
        (<<C as SingletonForeignKey>::ReferencedTable as diesel::Table>::PrimaryKey,),
    > for C
where
    <<C as SingletonForeignKey>::ReferencedTable as diesel::Table>::PrimaryKey: PrimaryKeyColumn,
    C: SingletonForeignKey,
{
}
