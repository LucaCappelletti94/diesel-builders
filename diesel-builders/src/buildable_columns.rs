//! Module for buildable columns in Diesel.

use diesel_additions::{Columns, TypedColumn};

use crate::{BuildableTable, buildable_tables::BuildableTables};

/// A trait for Diesel columns that can be built.
pub trait BuildableColumn: TypedColumn<Table: BuildableTable> {}

impl<C> BuildableColumn for C
where
    C: TypedColumn,
    C::Table: BuildableTable,
{
}

/// A trait for collections of Diesel columns that can be built.
pub trait BuildableColumns: Columns<Tables: BuildableTables> {}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_buildable_columns]
mod impls {}
