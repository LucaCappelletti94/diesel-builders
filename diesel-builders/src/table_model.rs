//! Submodule defining the `TableModel` trait.

use crate::{HasTableExt, TableExt};

/// A marker trait for Diesel table models.
///
/// This trait indicates that a type represents a Diesel table model and
/// provides access to the associated table type. It is automatically
/// implemented for any type that has a table with the required extensions.
///
/// This trait is typically derived automatically via the
/// `#[derive(TableModel)]` macro on your model structs.
pub trait TableModel: HasTableExt<Table: TableExt<Model = Self>> + Sized {}

impl<T> TableModel for T where T: HasTableExt<Table: TableExt<Model = T>> {}
