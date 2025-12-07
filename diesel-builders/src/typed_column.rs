//! Submodule providing the `Typed` trait.

use crate::{TableAddition, Typed};

/// Trait representing an object with an associated type.
pub trait TypedColumn: diesel::Column<Table: TableAddition> + Typed + Default {}

impl<T> TypedColumn for T where T: diesel::Column<Table: TableAddition> + Typed + Default {}
