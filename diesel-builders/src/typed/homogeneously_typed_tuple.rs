//! Submodule providing the `HomogeneouslyTypedTuple` trait.

use tuplities::prelude::NestTuple;

use crate::TypedTuple;

use super::HomogeneouslyTypedNestedTuple;

/// Trait representing a tuple where all entries implement Typed with the same associated Type.
pub trait HomogeneouslyTypedTuple<CT>: NestTuple + TypedTuple {}

impl<T, CT> HomogeneouslyTypedTuple<CT> for T where
    T: NestTuple<Nested: HomogeneouslyTypedNestedTuple<CT>> + TypedTuple
{
}
