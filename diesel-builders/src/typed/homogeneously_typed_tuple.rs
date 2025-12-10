//! Submodule providing the `HomogeneouslyTypedTuple` trait.

use tuplities::prelude::{NestTuple, TupleReplicate};

use crate::TypedTuple;

use super::HomogeneouslyTypedNestedTuple;

/// Trait representing a tuple where all entries implement Typed with the same associated Type.
pub trait HomogeneouslyTypedTuple<CT>:
    NestTuple + TypedTuple<TupleType: TupleReplicate<CT>>
{
}

impl<T, CT> HomogeneouslyTypedTuple<CT> for T where
    T: NestTuple<Nested: HomogeneouslyTypedNestedTuple<CT>>
        + TypedTuple<TupleType: TupleReplicate<CT>>
{
}
