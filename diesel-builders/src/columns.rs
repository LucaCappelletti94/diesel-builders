//! Submodule defining and implementing the `Columns` trait.

use crate::{OptionTuple, RefTuple, Tables, TypedColumn, tables::TableModels};

/// A trait representing a collection of Diesel columns.
pub trait Columns {
    /// Tuple of data types of the columns.
    type Types: OptionTuple + RefTuple;
    /// Tables to which these columns belong.
    type Tables: Tables<Models: TableModels<Tables = Self::Tables>>;
}

/// A trait representing a potentially empty projection of Diesel columns.
pub trait Projection<T: diesel::Table>: Columns {}

/// A trait representing a non-empty projection of Diesel columns.
pub trait NonEmptyProjection: Projection<Self::Table> {
    /// The table associated to this projection.
    type Table: diesel::Table;
}

/// A trait representing a collection of Diesel columns with an associated type.
pub trait HomogeneousColumns<Type>: Columns {}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_columns]
mod impls {}
