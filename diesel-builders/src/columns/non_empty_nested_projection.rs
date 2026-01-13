//! Submodule defining and implementing the `NonEmptyNestedProjection` trait.

use super::NestedColumns;
use crate::TypedColumn;

/// Trait for nested columns tuples that represent non-empty projections.
pub trait NonEmptyNestedProjection: NestedColumns {
    /// The table associated to this projection.
    type Table: diesel::Table + Default;
}

impl<C1: TypedColumn> NonEmptyNestedProjection for (C1,) {
    type Table = C1::Table;
}

impl<Head, Tail> NonEmptyNestedProjection for (Head, Tail)
where
    Head: TypedColumn,
    Tail: NonEmptyNestedProjection<Table = Head::Table>,
    (Head, Tail): NestedColumns,
{
    type Table = Head::Table;
}
