//! Submodule defining the `TableModel` trait.

use crate::{HasTableExt, TableExt};

/// Trait representing a Diesel table model.
pub trait TableModel: HasTableExt<Table: TableExt<Model = Self>> + Sized {}

impl<T> TableModel for T where T: HasTableExt<Table: TableExt<Model = T>> {}
