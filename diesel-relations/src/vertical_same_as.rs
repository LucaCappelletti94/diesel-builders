//! Submodule defining an `VerticalSameAs` trait for Diesel columns.

use diesel::{Column, Table};
use diesel_additions::ForeignKey;

/// A trait for Diesel columns that define vertical same-as relationships.
pub trait VerticalSameAs<AncestorColumn: Column>: Column {}

impl<AncestorColumn: Column, HostColumn: Column> VerticalSameAs<AncestorColumn> for HostColumn
where
    HostColumn: Column,
    <<AncestorColumn as Column>::Table as Table>::PrimaryKey:
        Column<Table = <AncestorColumn as Column>::Table>,
    <<HostColumn as Column>::Table as Table>::PrimaryKey:
        Column<Table = <HostColumn as Column>::Table>,
    (<<HostColumn as Column>::Table as Table>::PrimaryKey,):
        ForeignKey<(<<AncestorColumn as Column>::Table as Table>::PrimaryKey,)>,
    (<<HostColumn as Column>::Table as Table>::PrimaryKey, HostColumn):
        ForeignKey<(<<AncestorColumn as Column>::Table as Table>::PrimaryKey, AncestorColumn)>,
{
}

/// Trait marker for inferred vertical same-as relationships.
pub trait InferredVerticalSameAs<AncestorColumn: Column>: Column {}

impl<AncestorColumn: Column, T> InferredVerticalSameAs<AncestorColumn> for T where
    T: VerticalSameAs<AncestorColumn>
{
}
