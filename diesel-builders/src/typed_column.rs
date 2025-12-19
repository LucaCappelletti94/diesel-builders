//! Submodule providing the `Typed` trait.

use crate::Typed;

/// Trait representing an object with an associated type.
pub trait TypedColumn: diesel::Column<Table: Default> + Typed + Default + Copy {}

impl<T> TypedColumn for T where T: diesel::Column<Table: Default> + Typed + Default + Copy {}
