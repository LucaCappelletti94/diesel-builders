//! Submodule defining and implementing traits for Diesel tables.

mod has_nested_tables;
mod nested_tables;
mod non_composite_primary_key_tables;

pub use has_nested_tables::HasNestedTables;
pub use nested_tables::NestedTables;
pub use non_composite_primary_key_tables::{
    NonCompositePrimaryKeyNestedTables, NonCompositePrimaryKeyTables,
};
use tuplities::prelude::NestTuple;

/// A trait representing a collection of Diesel tables.
pub trait Tables: NestTuple {}

impl<T> Tables for T where T: NestTuple<Nested: NestedTables<Flattened = T>> {}
