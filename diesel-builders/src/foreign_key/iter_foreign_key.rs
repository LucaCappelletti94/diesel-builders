//! Submodule defining a trait to iterate the foreign keys in a table
//! which reference the same foreign index in another table.

use crate::columns::NonEmptyProjection;

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
    type ForeignKeysIter<'a>: Iterator
    where
        Idx: 'a,
        Self: 'a;

    /// Foreign key n-uple Iterator item type, most commonly an
    /// n-uple of `Box<dyn DynTypedColumn<ValueType=ith column of index
    /// ValueType>>`.
    type ForeignKeyItemType;

    /// Iterator yield the tuples of column values corresponding to all foreign
    /// keys in this table. Most commonly an n-uple of `Box<dyn
    /// DynTypedColumn<ValueType=ith column of index ValueType>>`.
    type ForeignKeyColumnsIter: Iterator<Item = Self::ForeignKeyItemType>;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    fn iter_foreign_keys(&self) -> Self::ForeignKeysIter<'_>;

    /// Returns an iterator over the foreign keys in this table.
    fn iter_foreign_key_columns(&self) -> Self::ForeignKeyColumnsIter;
}

/// An extension of the `IterForeignKey` trait moving the generic parameter
/// from the trait to the method to facilitate usage in certain contexts.
pub trait IterForeignKeyExt {
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    fn iter_foreign_keys<Idx>(&self) -> <Self as IterForeignKey<Idx>>::ForeignKeysIter<'_>
    where
        Idx: NonEmptyProjection,
        Self: IterForeignKey<Idx>,
    {
        IterForeignKey::iter_foreign_keys(self)
    }

    /// Returns an iterator over the foreign keys in this table.
    fn iter_foreign_key_columns<Idx>(&self) -> <Self as IterForeignKey<Idx>>::ForeignKeyColumnsIter
    where
        Idx: NonEmptyProjection,
        Self: IterForeignKey<Idx>,
    {
        IterForeignKey::iter_foreign_key_columns(self)
    }
}

impl<T> IterForeignKeyExt for T {}
