//! Submodule defining the `Descendant` trait.

use diesel_additions::{TableAddition, Tables};
use typed_tuple::{ChainTuple, TypedLast};

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: TableAddition {
    /// The ancestor tables of this table.
    type Ancestors: Tables;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant {
    /// The ancestor tables of this table, including itself.
    type AncestorsWithSelf: Tables + TypedLast<Self>;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant,
    T::Ancestors: ChainTuple<(T,)>,
    <T::Ancestors as ChainTuple<(T,)>>::Output: Tables + TypedLast<T>,
{
    type AncestorsWithSelf = <T::Ancestors as ChainTuple<(T,)>>::Output;
}
