//! Module for buildable columns in Diesel.

use tuple_set::TupleSet;

use crate::Tables;
use tuplities::prelude::*;

/// A trait for collections of Diesel tables that can be built.
pub trait BuildableTables: Tables {
    /// The builders associated with the buildable tables.
    type Builders: IntoTupleOption<IntoOptions = Self::OptionalBuilders> + TupleSet;
    /// The optional builders associated with the buildable tables.
    type OptionalBuilders: TupleOption<Transposed = Self::Builders>;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_buildable_tables]
mod impls {}
