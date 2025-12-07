//! Module for buildable columns in Diesel.

use crate::tables::TablesExt;
use tuplities::prelude::*;

#[diesel_builders_macros::impl_buildable_tables]
/// A trait for collections of Diesel tables that can be built.
/// Limited to 8 tables as building complex hierarchies with more
/// tables leads to performance issues.
pub trait BuildableTables: TablesExt {
    /// The builders associated with the buildable tables.
    type Builders: IntoTupleOption<IntoOptions = Self::OptionalBuilders>;
    /// The optional builders associated with the buildable tables.
    type OptionalBuilders: TupleOption<Transposed = Self::Builders>;
}
