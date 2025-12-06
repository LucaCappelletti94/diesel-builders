//! Submodule providing the implementations of traits from the core library for table builders.

use core::fmt::Debug;

use tuplities::prelude::*;

use crate::{BuildableTable, BundlableTables, TableBuilder};

impl<T: BuildableTable> Clone for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleClone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            bundles: self.bundles.tuple_clone(),
        }
    }
}

impl<T: BuildableTable> Copy for TableBuilder<T> where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: Copy + TupleCopy
{
}

impl<T: BuildableTable> PartialEq for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TuplePartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.bundles.tuple_eq(&other.bundles)
    }
}

impl<T: BuildableTable> Eq for TableBuilder<T> where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleEq
{
}

impl<T: BuildableTable> core::hash::Hash for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleHash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.bundles.tuple_hash(state);
    }
}

impl<T: BuildableTable> PartialOrd for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TuplePartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.bundles.tuple_partial_cmp(&other.bundles)
    }
}

impl<T: BuildableTable> Ord for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleOrd,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.bundles.tuple_cmp(&other.bundles)
    }
}

impl<T: BuildableTable> Debug for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: TupleDebug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilder")
            .field("bundles", &self.bundles.tuple_debug())
            .finish()
    }
}

impl<T: BuildableTable> Default for TableBuilder<T> {
    #[inline]
    fn default() -> Self {
        Self {
            bundles: TupleDefault::tuple_default(),
        }
    }
}
