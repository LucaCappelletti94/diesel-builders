//! Submodule defining and implementing traits for Diesel columns.

mod columns_collection;
mod homogeneously_typed_nested_columns;
mod nested_columns;
mod nested_columns_collection;
mod non_empty_nested_projection;
mod non_empty_projection;
mod tuple_eq_all;

pub use columns_collection::ColumnsCollection;
pub use homogeneously_typed_nested_columns::HomogeneouslyTypedNestedColumns;
pub use nested_columns::NestedColumns;
pub use nested_columns_collection::NestedColumnsCollection;
pub use non_empty_nested_projection::NonEmptyNestedProjection;
pub use non_empty_projection::NonEmptyProjection;
pub use tuple_eq_all::TupleEqAll;

use crate::TypedTuple;
use tuplities::prelude::*;

/// A trait representing a collection of Diesel columns.
pub trait Columns: TypedTuple<Nested: Default> {
    /// Tables to which these columns belong.
    type Tables: NestTuple;
}

impl<T> Columns for T
where
    T: TypedTuple + NestTuple<Nested: NestedColumns>,
    <<T::Nested as NestedColumns>::NestedTables as FlattenNestedTuple>::Flattened: NestTuple,
{
    type Tables = <<T::Nested as NestedColumns>::NestedTables as FlattenNestedTuple>::Flattened;
}
