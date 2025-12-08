//! Submodule defining the `Descendant` trait.

use tuplities::prelude::{TuplePopFront, TuplePushBack};
use typenum::Unsigned;

use crate::{BundlableTables, TableAddition, Tables};

/// Marker trait for root table models (tables with no ancestors).
///
/// This trait should be derived on Model structs to automatically generate
/// the `Descendant` implementation for their associated table type.
pub trait Root: crate::TableAddition {}

/// A trait marker for getting the ancestor index of a table.
pub trait AncestorOfIndex<T: DescendantOf<Self>>: TableAddition + DescendantOf<T::Root> {
    /// Tuple index marker of the ancestor table in the descendant's ancestor
    /// list.
    type Idx: Unsigned;
}

/// A trait for Diesel tables that have ancestor tables.
pub trait DescendantOf<T: TableAddition>: Descendant {}

impl<T> DescendantOf<T> for T where T: Descendant {}

/// A trait marker for getting the ancestor tables of a descendant table.
pub trait AncestorsOf<T: Descendant<Ancestors = Self>>: Tables {}

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: TableAddition {
    /// The ancestor tables of this table.
    type Ancestors: AncestorsOf<Self> + TuplePushBack<Self>;
    /// The root of the ancestor hierarchy. When the current
    /// table is the root, this is itself.
    type Root: Root;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant {
    /// The ancestor tables of this table, including itself.
    type AncestorsWithSelf: BundlableTables + TuplePopFront<Front = Self::Root>;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant,
    T::Ancestors: TuplePushBack<Self>,
    <T::Ancestors as TuplePushBack<Self>>::Output: BundlableTables + TuplePopFront<Front = T::Root>,
{
    type AncestorsWithSelf = <T::Ancestors as TuplePushBack<Self>>::Output;
}

/// Implementation of `AncestorsOf` trait for tuples.
/// Limited to 8 ancestors as deep inheritance hierarchies beyond
/// this point are problematic for query performance and maintainability.
#[diesel_builders_macros::impl_ancestors_of]
mod impls {}
