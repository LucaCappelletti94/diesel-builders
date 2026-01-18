//! Blanket implementations of `IterForeignKey` for tuples.

use tuplities::prelude::{IntoNestedTupleOption, NestedTupleRef};

use crate::{IterForeignKey, NestedColumns, TypedNestedTuple, columns::NonEmptyProjection};

impl<Idx: NonEmptyProjection> IterForeignKey<Idx> for () {
    type MatchSimpleIter<'a> = std::iter::Empty<<<<Idx::Nested as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions>
        where
            Idx: 'a,
            Self: 'a;

    type MatchFullIter<'a>
        = std::iter::Empty<
        <<Idx::Nested as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a>,
    >
    where
        Idx: 'a,
        Self: 'a;

    type ForeignKeyColumnsIter = std::iter::Empty<<Idx::Nested as NestedColumns>::DynColumns>;

    #[inline]
    fn iter_match_simple<'a>(&'a self) -> Self::MatchSimpleIter<'a>
    where
        Idx: 'a,
    {
        std::iter::empty()
    }

    #[inline]
    fn iter_match_full<'a>(&'a self) -> Self::MatchFullIter<'a>
    where
        Idx: 'a,
    {
        std::iter::empty()
    }

    #[inline]
    fn iter_foreign_key_columns() -> Self::ForeignKeyColumnsIter {
        std::iter::empty()
    }
}

impl<Idx: NonEmptyProjection, T> IterForeignKey<Idx> for (T,)
where
    T: IterForeignKey<Idx>,
{
    type MatchSimpleIter<'a>
        = T::MatchSimpleIter<'a>
    where
        Idx: 'a,
        Self: 'a;

    type MatchFullIter<'a>
        = T::MatchFullIter<'a>
    where
        Idx: 'a,
        Self: 'a;

    type ForeignKeyColumnsIter = T::ForeignKeyColumnsIter;

    #[inline]
    fn iter_match_simple<'a>(&'a self) -> Self::MatchSimpleIter<'a>
    where
        Idx: 'a,
    {
        self.0.iter_match_simple()
    }

    #[inline]
    fn iter_match_full<'a>(&'a self) -> Self::MatchFullIter<'a>
    where
        Idx: 'a,
    {
        self.0.iter_match_full()
    }

    #[inline]
    fn iter_foreign_key_columns() -> Self::ForeignKeyColumnsIter {
        T::iter_foreign_key_columns()
    }
}

impl<Idx: NonEmptyProjection, Head, Tail> IterForeignKey<Idx> for (Head, Tail)
where
    Head: IterForeignKey<Idx>,
    Tail: IterForeignKey<Idx>,
{
    type MatchSimpleIter<'a>
        = std::iter::Chain<Head::MatchSimpleIter<'a>, Tail::MatchSimpleIter<'a>>
    where
        Idx: 'a,
        Self: 'a;

    type MatchFullIter<'a>
        = std::iter::Chain<Head::MatchFullIter<'a>, Tail::MatchFullIter<'a>>
    where
        Idx: 'a,
        Self: 'a;

    type ForeignKeyColumnsIter =
        std::iter::Chain<Head::ForeignKeyColumnsIter, Tail::ForeignKeyColumnsIter>;

    #[inline]
    fn iter_match_simple<'a>(&'a self) -> Self::MatchSimpleIter<'a>
    where
        Idx: 'a,
    {
        self.0.iter_match_simple().chain(self.1.iter_match_simple())
    }

    #[inline]
    fn iter_match_full<'a>(&'a self) -> Self::MatchFullIter<'a>
    where
        Idx: 'a,
    {
        self.0.iter_match_full().chain(self.1.iter_match_full())
    }

    #[inline]
    fn iter_foreign_key_columns() -> Self::ForeignKeyColumnsIter {
        std::iter::chain(Head::iter_foreign_key_columns(), Tail::iter_foreign_key_columns())
    }
}
