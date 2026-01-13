//! Submodule defining a trait to iterate the foreign keys in a table
//! which reference the same foreign index in another table.

use tuplities::prelude::{FlattenNestedTuple, NestTuple, NestTupleRef};

use crate::{TypedNestedTuple, UniqueTableIndex};

/// An iterator over foreign keys in a table which reference the same foreign
/// index in another table. If the values of the foreign keys are options,
/// possible `None` values are filtered out.
///
/// This trait does NOT require a `Conn` type parameter, as it only operates on
/// the in-memory representation of the table model. It does not query the
/// database.
pub trait IterForeignKey<
    Idx: UniqueTableIndex<Nested: TypedNestedTuple<NestedTupleValueType: NestTupleRef>>,
>
{
    /// The iterator constructed by this trait, which must yield the tuples of
    /// column values corresponding to the foreign keys.
    type ForeignKeysIter<'a>: Iterator<
        Item = <<<<Idx as NestTuple>::Nested as TypedNestedTuple>::NestedTupleValueType as NestTupleRef>::NestedRef<'a> as FlattenNestedTuple>::Flattened,
    > where
		Idx: 'a,
		Self: 'a;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index.
    fn iter_foreign_key(&self) -> Self::ForeignKeysIter<'_>;
}

/// An extension of the `IterForeignKey` trait moving the generic parameter
/// from the trait to the method to facilitate usage in certain contexts.
pub trait IterForeignKeyExt {
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index.
    fn iter_foreign_key<Idx>(&self) -> <Self as IterForeignKey<Idx>>::ForeignKeysIter<'_>
    where
        Idx: UniqueTableIndex<Nested: TypedNestedTuple<NestedTupleValueType: NestTupleRef>>,
        Self: IterForeignKey<Idx>,
    {
        IterForeignKey::iter_foreign_key(self)
    }
}

impl<T> IterForeignKeyExt for T {}
