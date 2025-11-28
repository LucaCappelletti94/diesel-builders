//! Module for buildable columns in Diesel.

use diesel_additions::{ClonableTuple, DebuggableTuple, OptionTuple, Tables};

use crate::{BuildableTable, TableBuilder};

/// A trait for collections of Diesel tables that can be built.
pub trait BuildableTables: Tables {
    /// The builders associated with the buildable tables.
    type Builders: OptionTuple + ClonableTuple + DebuggableTuple;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_buildable_tables]
mod impls {}
