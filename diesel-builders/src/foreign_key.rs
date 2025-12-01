//! Submodule defining a `ForeignKey` trait for Diesel tables.

use typed_tuple::prelude::{NthIndex, TypedIndex, U0, Unsigned};

use crate::{
    NonCompositePrimaryKeyTableModel, TypedColumn, columns::NonEmptyProjection,
    table_addition::HasPrimaryKey,
};

/// A trait for Diesel tables that define indices which
/// can be used by foreign keys.
#[diesel_builders_macros::impl_table_index]
pub trait TableIndex: NonEmptyProjection {}

/// A trait for Diesel columns which are part of a `TableIndex`.
pub trait IndexedColumn<Idx: Unsigned, IndexedColumns: TableIndex + TypedIndex<Idx, Self>>:
    TypedColumn
{
}

/// A trait for Diesel tables that define foreign key relationships.
#[diesel_builders_macros::impl_foreign_key]
pub trait ForeignKey<ReferencedColumns: TableIndex>: NonEmptyProjection {}

/// A trait for Diesel columns that are part of a foreign key relationship.
///
/// This trait should be implemented for each column in a foreign key tuple.
/// Use the `fk!` macro to implement this trait.
pub trait HostColumn<
    Idx: Unsigned,
    HostColumns: ForeignKey<ReferencedColumns> + TypedIndex<Idx, Self>,
    ReferencedColumns: TableIndex + NthIndex<Idx, NthType: TypedColumn<Type = <Self as TypedColumn>::Type>>,
>: TypedColumn
{
}

/// A trait for Diesel columns that define single-column foreign key
/// relationships to tables with a singleton primary key.
pub trait SingletonForeignKey: TypedColumn {
    /// The referenced table.
    type ReferencedTable: HasPrimaryKey<
            PrimaryKey: TypedColumn<Type = <Self as TypedColumn>::Type>,
            Model: NonCompositePrimaryKeyTableModel,
        > + diesel::query_source::TableNotEqual<Self::Table>;
}

impl<C>
    HostColumn<
        U0,
        (C,),
        (<<C as SingletonForeignKey>::ReferencedTable as diesel::Table>::PrimaryKey,),
    > for C
where
    C: SingletonForeignKey,
{
}
