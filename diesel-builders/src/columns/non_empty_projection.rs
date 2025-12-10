//! Submodule defining and implementing the `NonEmptyProjection` trait.

use super::{Columns, NestedColumns, NonEmptyNestedProjection};

/// A trait representing a non-empty projection of Diesel columns.
pub trait NonEmptyProjection: Columns<Nested: NestedColumns> {
    /// The table associated to this projection.
    type Table: diesel::Table + Default;
}

impl<T> NonEmptyProjection for T
where
    T: Columns<Nested: NonEmptyNestedProjection>,
{
    type Table = <T::Nested as NonEmptyNestedProjection>::Table;
}
