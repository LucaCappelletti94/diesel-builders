//! Submodule defining a trait to iterate the foreign keys in a table
//! which reference the same foreign index in another table.

use tuplities::prelude::{IntoNestedTupleOption, NestedTupleRef};

use crate::{NestedColumns, TypedNestedTuple, columns::NonEmptyProjection};

/// An iterator over foreign keys in a table which reference the same foreign
/// index in another table.
///
/// This trait does NOT require a `Conn` type parameter, as it only operates on
/// the in-memory representation of the table model. It does not query the
/// database.
pub trait IterForeignKey<Idx: NonEmptyProjection> {
    /// Iterator yielding the tuples of column values corresponding to the
    /// foreign keys. When the foreign key contains any `None`, those keys are
    /// NOT skipped.
    type MatchSimpleIter<'a>: Iterator<
        Item = <<<Idx::Nested as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a> as IntoNestedTupleOption>::IntoOptions
    >
    where
        Idx: 'a,
        Self: 'a;

    /// Iterator yielding the tuples of column values corresponding to the
    /// foreign keys. When the foreign key contains any `None`, those keys
    /// are skipped.
    type MatchFullIter<'a>: Iterator<
        Item = <<Idx::Nested as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a>,
    >
    where
        Idx: 'a,
        Self: 'a;

    /// Iterator yield the tuples of column values corresponding to all foreign
    /// keys in this table.
    type ForeignKeyColumnsIter: Iterator<Item = <Idx::Nested as NestedColumns>::DynColumns>;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    fn iter_match_simple<'a>(&'a self) -> Self::MatchSimpleIter<'a>
    where
        Idx: 'a;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are skipped.
    fn iter_match_full<'a>(&'a self) -> Self::MatchFullIter<'a>
    where
        Idx: 'a;

    /// Returns an iterator over the foreign keys in this table.
    fn iter_foreign_key_columns() -> Self::ForeignKeyColumnsIter;
}

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

/// An extension of the `IterForeignKey` trait moving the generic parameter
/// from the trait to the method to facilitate usage in certain contexts.
pub trait IterForeignKeyExt {
    #[inline]
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    fn iter_match_simple<Idx>(&self) -> <Self as IterForeignKey<Idx>>::MatchSimpleIter<'_>
    where
        Idx: NonEmptyProjection,
        Self: IterForeignKey<Idx>,
    {
        IterForeignKey::iter_match_simple(self)
    }

    #[inline]
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are skipped.
    fn iter_match_full<Idx>(&self) -> <Self as IterForeignKey<Idx>>::MatchFullIter<'_>
    where
        Idx: NonEmptyProjection,
        Self: IterForeignKey<Idx>,
    {
        IterForeignKey::iter_match_full(self)
    }

    #[must_use]
    #[inline]
    /// Returns an iterator over the foreign keys in this table.
    fn iter_foreign_key_columns<Idx>() -> <Self as IterForeignKey<Idx>>::ForeignKeyColumnsIter
    where
        Idx: NonEmptyProjection,
        Self: IterForeignKey<Idx>,
    {
        <Self as IterForeignKey<Idx>>::iter_foreign_key_columns()
    }
}

impl<T> IterForeignKeyExt for T {}
