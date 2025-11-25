//! Submodule defining the `Descendant` trait.

use diesel_additions::{TableAddition, Tables};
use typed_tuple::{ChainRight, TypedLast, TypedFirst};

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: TableAddition {
    /// The ancestor tables of this table.
    type Ancestors: Tables;
    /// The root of the ancestor hierarchy. When the current
    /// table is the root, this is itself.
    type Root: TableAddition;
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
