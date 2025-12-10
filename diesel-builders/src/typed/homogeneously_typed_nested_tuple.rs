//! Submodule defining and implementing the `HomogeneouslyTypedNestedTuple` trait.

use crate::{Typed, TypedNestedTuple};

/// Trait for recursive definition of homogeneous typed tuples.
/// All entries must have the same associated Type.
pub trait HomogeneouslyTypedNestedTuple<CT>: TypedNestedTuple {}

impl<CT> HomogeneouslyTypedNestedTuple<CT> for () {}

impl<T> HomogeneouslyTypedNestedTuple<T::Type> for (T,) where T: Typed {}

impl<Head, Tail> HomogeneouslyTypedNestedTuple<Head::Type> for (Head, Tail)
where
    Head: Typed,
    Tail: HomogeneouslyTypedNestedTuple<Head::Type>,
    Self: TypedNestedTuple<NestedTupleType = (Head::Type, Tail::NestedTupleType)>,
{
}
