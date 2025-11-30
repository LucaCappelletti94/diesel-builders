//! Module for buildable columns in Diesel.

use tuple_set::TupleSet;

use crate::{BuildableTable, ClonableTuple, DebuggableTuple, OptionTuple, TableBuilder, Tables};

/// A trait for collections of Diesel tables that can be built.
pub trait BuildableTables: Tables {
    /// The builders associated with the buildable tables.
    type Builders: OptionTuple + ClonableTuple + DebuggableTuple + TupleSet;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_buildable_tables]
mod impls {}
