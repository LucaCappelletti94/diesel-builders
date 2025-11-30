//! Submodule defining an `VerticalSameAs` trait for Diesel columns.

use crate::{TableInherits, TypedColumn, table_addition::HasPrimaryKey};

/// A trait for Diesel columns that define vertical same-as relationships.
pub trait VerticalSameAs<AncestorColumn: TypedColumn<Table: HasPrimaryKey>>:
    TypedColumn<Table: TableInherits<AncestorColumn::Table>>
{
}

impl<AncestorColumn, HostColumn> VerticalSameAs<AncestorColumn> for HostColumn
where
    AncestorColumn: TypedColumn<Table: HasPrimaryKey>,
    HostColumn: TypedColumn<Table: TableInherits<AncestorColumn::Table>>,
{
}

/// Trait marker for inferred vertical same-as relationships.
pub trait InferredVerticalSameAs<AncestorColumn: TypedColumn<Table: HasPrimaryKey>>:
    TypedColumn<Table: HasPrimaryKey>
{
}

impl<AncestorColumn: TypedColumn<Table: HasPrimaryKey>, T> InferredVerticalSameAs<AncestorColumn>
    for T
where
    T: VerticalSameAs<AncestorColumn>,
{
}
