//! Submodule defining the `BundlableTables` trait, which defines an n-tuple of
//! Diesel tables that implement the `BundlableTable` trait.

use tuplities::prelude::{FlattenNestedTuple, NestedTupleTryFrom};

use crate::{
    CompletedTableBuilderBundle, IncompleteBuilderError, TableBuilderBundle,
    builder_bundle::BundlableTableExt,
    construction::{BuildDefaults, DefaultBundle},
    tables::NestedTables,
};

/// A trait for collections of Diesel tables that can be used in table builder
/// bundles.
pub trait NestedBundlableTables: NestedTables {
    /// The bundles of table builders for the buildable tables.
    type NestedBundleBuilders: FlattenNestedTuple;
    /// The completed bundles of table builders for the buildable tables.
    type NestedCompletedBundleBuilders: FlattenNestedTuple
        + NestedTupleTryFrom<Self::NestedBundleBuilders, IncompleteBuilderError>;
    /// The staged default bundles for the buildable tables, matching the
    /// ancestor indexing of the ordinary builder bundles.
    type NestedDefaultBundleBuilders: BuildDefaults<CheckedBundles = Self::NestedBundleBuilders>
        + Default;
}

impl NestedBundlableTables for () {
    type NestedBundleBuilders = ();
    type NestedCompletedBundleBuilders = ();
    type NestedDefaultBundleBuilders = ();
}

impl<T1> NestedBundlableTables for (T1,)
where
    T1: BundlableTableExt,
    <T1 as BundlableTableExt>::OptionalMandatoryNestedBuilders: Default,
    <T1 as BundlableTableExt>::OptionalDiscretionaryNestedBuilders: Default,
    DefaultBundle<T1>: BuildDefaults<CheckedBundles = TableBuilderBundle<T1>>,
{
    type NestedBundleBuilders = (TableBuilderBundle<T1>,);
    type NestedCompletedBundleBuilders = (CompletedTableBuilderBundle<T1>,);
    type NestedDefaultBundleBuilders = (DefaultBundle<T1>,);
}

impl<Thead, Ttail> NestedBundlableTables for (Thead, Ttail)
where
    Thead: BundlableTableExt,
    <Thead as BundlableTableExt>::OptionalMandatoryNestedBuilders: Default,
    <Thead as BundlableTableExt>::OptionalDiscretionaryNestedBuilders: Default,
    Ttail: NestedBundlableTables,
    (Thead, Ttail): NestedTables,
    (TableBuilderBundle<Thead>, Ttail::NestedBundleBuilders): FlattenNestedTuple,
    (CompletedTableBuilderBundle<Thead>, Ttail::NestedCompletedBundleBuilders): FlattenNestedTuple,
    DefaultBundle<Thead>: BuildDefaults<CheckedBundles = TableBuilderBundle<Thead>>,
{
    type NestedBundleBuilders = (TableBuilderBundle<Thead>, Ttail::NestedBundleBuilders);
    type NestedCompletedBundleBuilders =
        (CompletedTableBuilderBundle<Thead>, Ttail::NestedCompletedBundleBuilders);
    type NestedDefaultBundleBuilders = (DefaultBundle<Thead>, Ttail::NestedDefaultBundleBuilders);
}
