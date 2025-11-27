//! Submodule defining a `ForeignKey` trait for Diesel tables.

use diesel::Column;

use crate::{
    NonCompositePrimaryKeyTableModel, Projection, TypedColumn, table_addition::HasPrimaryKey,
};

/// A trait for Diesel tables that define foreign key relationships.
pub trait ForeignKey<ReferencedColumns: Projection>: Projection {}

/// A trait for Diesel columns that define single-column foreign key
/// relationships.
pub trait SingleColumnForeignKey<ReferencedColumn: Column>: Column {}

impl<HostColumn, ReferencedColumn> SingleColumnForeignKey<ReferencedColumn> for HostColumn
where
    HostColumn: Column,
    ReferencedColumn: Column,
    (ReferencedColumn,): ForeignKey<(ReferencedColumn,)>,
{
}

/// A trait for Diesel columns that define single-column foreign key
/// relationships to tables with a singleton primary key.
pub trait SingletonForeignKey: TypedColumn {
    /// The referenced table.
    type ReferencedTable: HasPrimaryKey<
            PrimaryKey: TypedColumn<Type = <Self as TypedColumn>::Type>,
            Model: NonCompositePrimaryKeyTableModel,
        >;
}
