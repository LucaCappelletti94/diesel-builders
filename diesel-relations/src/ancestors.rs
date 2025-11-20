//! Submodule defining the `Descendant` trait.

use diesel_additions::{ExtendTuple, Tables};

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: diesel::Table + Sized {
    /// The unique ancestor tables of this table.
    type UniqueAncestors: Tables;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant {
    /// The unique ancestor tables of this table, including itself.
    type UniqueAncestorsWithSelf: Tables;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant,
    T::UniqueAncestors: ExtendTuple<(T,)>,
    <T::UniqueAncestors as ExtendTuple<(T,)>>::Output: Tables,
{
    type UniqueAncestorsWithSelf = <T::UniqueAncestors as ExtendTuple<(T,)>>::Output;
}
