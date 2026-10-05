//! Submodule defining and implementing the `ColumnsCollection` trait.

use tuplities::prelude::*;

use super::NestedColumnsCollection;

/// A trait representing a matrix (a collection of collections) of Diesel
/// columns.
pub trait ColumnsCollection: NestTupleMatrix {}

impl<C> ColumnsCollection for C where
    C: NestTupleMatrix<NestedMatrix: NestedColumnsCollection<FlattenedMatrix = Self>>
{
}
