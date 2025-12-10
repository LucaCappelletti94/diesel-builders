//! Submodule defining the `Descendant` trait.

use tuplities::prelude::{FlattenNestedTuple, TuplePopFront, TuplePushBack};
use typenum::Unsigned;

use crate::{NestedBundlableTables, TableExt, Tables, tables::NestedTables};

/// Marker trait for root table models (tables with no ancestors).
///
/// This trait should be derived on Model structs to automatically generate
/// the `Descendant` implementation for their associated table type.
pub trait Root: crate::TableExt {}

/// A trait marker for getting the ancestor index of a table.
pub trait AncestorOfIndex<T: DescendantOf<Self>>: TableExt + DescendantOf<T::Root> {
    /// Tuple index marker of the ancestor table in the descendant's ancestor
    /// list.
    type Idx: Unsigned;
}

/// A trait for Diesel tables that have ancestor tables.
pub trait DescendantOf<T: TableExt>: Descendant {}

impl<T> DescendantOf<T> for T where T: Descendant {}

/// A trait marker for getting the ancestor tables of a descendant table.
pub trait NestedAncestorsOf<T: Descendant<Ancestors = <Self as FlattenNestedTuple>::Flattened>>:
    NestedTables
{
}

/// A trait for Diesel tables that have ancestor tables.
pub trait Descendant: TableExt {
    /// The ancestor tables of this table.
    type Ancestors: TuplePushBack<Self> + Tables<Nested: NestedAncestorsOf<Self>>;
    /// The root of the ancestor hierarchy. When the current
    /// table is the root, this is itself.
    type Root: Root;
}

/// A trait for Diesel tables that have ancestor tables, including themselves.
pub trait DescendantWithSelf: Descendant {
    /// The ancestor tables of this table, including itself.
    type AncestorsWithSelf: TuplePopFront<Front = Self::Root>
        + Tables<Nested: NestedBundlableTables>;
}

impl<T> DescendantWithSelf for T
where
    T: Descendant,
    T::Ancestors: TuplePushBack<Self>,
    <T::Ancestors as TuplePushBack<Self>>::Output:
        Tables<Nested: NestedBundlableTables> + TuplePopFront<Front = T::Root>,
{
    type AncestorsWithSelf = <T::Ancestors as TuplePushBack<Self>>::Output;
}

impl<T> NestedAncestorsOf<T> for () where T: Descendant<Ancestors = ()> {}

impl<T, A> NestedAncestorsOf<T> for (A,)
where
    T: Descendant<Ancestors = (A,)>,
    A: AncestorOfIndex<T>,
    T: DescendantOf<A>,
    T: diesel::query_source::TableNotEqual<A>,
{
}

impl<T, Head, Tail> NestedAncestorsOf<T> for (Head, Tail)
where
    (Head, Tail): NestedTables,
    T: Descendant<Ancestors = <(Head, Tail) as FlattenNestedTuple>::Flattened>,
    Head: AncestorOfIndex<T>,
    T: DescendantOf<Head>,
    T: diesel::query_source::TableNotEqual<Head>,
{
}
