//! Module for buildable columns in Diesel.

use diesel_additions::{OptionTuple, Tables};

use crate::{
    BuildableTable, BuilderBundles, CompletedTableBuilderBundle, TableBuilder, TableBuilderBundle,
};

/// A trait for collections of Diesel tables that can be built.
pub trait BuildableTables: Tables {
    /// The builders associated with the buildable tables.
    type Builders: OptionTuple;
    /// The bundles of table builders for the buildable tables.
    type BuilderBundles: BuilderBundles<CompletedBundles = Self::CompletedBuilderBundles>;
    /// The completed bundles of table builders for the buildable tables.
    type CompletedBuilderBundles;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_buildable_tables]
mod impls {}
