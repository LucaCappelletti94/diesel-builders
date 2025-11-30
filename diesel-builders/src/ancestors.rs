//! Submodule defining the `Descendant` trait.

use typed_tuple::prelude::{ChainRight, TypedFirst, TypedLast, Unsigned};

use crate::{TableAddition, Tables};

/// Marker trait for root table models (tables with no ancestors).
///
/// This trait should be derived on Model structs to automatically generate
/// the `Descendant` implementation for their associated table type.
pub trait Root: crate::HasTableAddition {}

/// A trait marker for getting the ancestor index of a table.
pub trait AncestorOfIndex<T: DescendantOf<Self>>: TableAddition + DescendantOf<T::Root> {
    /// Tuple index marker of the ancestor table in the descendant's ancestor
    /// list.
    type Idx: Unsigned;
}

/// A trait for Diesel tables that have ancestor tables.
pub trait DescendantOf<T>: Descendant {}

impl<T> DescendantOf<T> for T where T: Descendant {}

/// A trait marker for getting the ancestor tables of a descendant table.
pub trait AncestorsOf<T: Descendant<Ancestors = Self>>: Tables {}

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: TableAddition {
    /// The ancestor tables of this table.
    type Ancestors: AncestorsOf<Self>;
    /// The root of the ancestor hierarchy. When the current
    /// table is the root, this is itself.
    type Root: Root;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant {
    /// The ancestor tables of this table, including itself.
    type AncestorsWithSelf: Tables + TypedLast<Self> + TypedFirst<Self::Root>;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant,
    T::Ancestors: ChainRight<(T,)>,
    <T::Ancestors as ChainRight<(T,)>>::Output: Tables + TypedLast<T> + TypedFirst<T::Root>,
{
    type AncestorsWithSelf = <T::Ancestors as ChainRight<(T,)>>::Output;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_ancestors_of]
mod impls {}
