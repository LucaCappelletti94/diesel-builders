//! Submodule defining the `BundlableTables` trait, which defines an n-tuple of
//! Diesel tables that implement the `BundlableTable` trait.

use crate::{
    BuilderBundles, ClonableTuple, CompletedTableBuilderBundle, TableBuilderBundle,
};

/// A trait for collections of Diesel tables that can be used in table builder
/// bundles.
pub trait BundlableTables {
    /// The bundles of table builders for the buildable tables.
    type BuilderBundles: BuilderBundles<CompletedBundles = Self::CompletedBuilderBundles>;
    /// The completed bundles of table builders for the buildable tables.
    type CompletedBuilderBundles: ClonableTuple;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_bundlable_tables]
mod impls {}
