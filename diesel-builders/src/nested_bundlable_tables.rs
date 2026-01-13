//! Submodule defining the `BundlableTables` trait, which defines an n-tuple of
//! Diesel tables that implement the `BundlableTable` trait.

use tuplities::prelude::{FlattenNestedTuple, NestedTupleTryFrom};

use crate::{
    CompletedTableBuilderBundle, IncompleteBuilderError, TableBuilderBundle,
    builder_bundle::BundlableTableExt, tables::NestedTables,
};

/// A trait for collections of Diesel tables that can be used in table builder
/// bundles.
pub trait NestedBundlableTables: NestedTables {
    /// The bundles of table builders for the buildable tables.
    type NestedBundleBuilders: Default + FlattenNestedTuple;
    /// The completed bundles of table builders for the buildable tables.
    type NestedCompletedBundleBuilders: FlattenNestedTuple
        + NestedTupleTryFrom<Self::NestedBundleBuilders, IncompleteBuilderError>;
}

impl NestedBundlableTables for () {
    type NestedBundleBuilders = ();
    type NestedCompletedBundleBuilders = ();
}

impl<T1> NestedBundlableTables for (T1,)
where
    T1: BundlableTableExt,
    <T1 as BundlableTableExt>::OptionalMandatoryNestedBuilders: Default,
    <T1 as BundlableTableExt>::OptionalDiscretionaryNestedBuilders: Default,
{
    type NestedBundleBuilders = (TableBuilderBundle<T1>,);
    type NestedCompletedBundleBuilders = (CompletedTableBuilderBundle<T1>,);
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
{
    type NestedBundleBuilders = (TableBuilderBundle<Thead>, Ttail::NestedBundleBuilders);
    type NestedCompletedBundleBuilders =
        (CompletedTableBuilderBundle<Thead>, Ttail::NestedCompletedBundleBuilders);
}
