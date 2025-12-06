//! Submodule defining and implementing the `Tables` trait.

use diesel::associations::HasTable;

use crate::{Columns, HomogeneousColumns, InsertableTableModel, TableAddition, TableModel};
use tuplities::prelude::*;

/// A trait representing a collection of Diesel tables.
pub trait Tables {
    /// The n-uple of models corresponding to the tables in this collection.
    type Models: TableModels<Tables = Self>;
    /// The n-uple of insertable models corresponding to the tables in this
    /// collection.
    type InsertableModels: InsertableTableModels<Tables = Self>;
    /// The primary keys of the tables in this collection.
    type PrimaryKeys;
}

/// A trait representing a collection of Diesel tables which
/// have non-composite primary keys.
pub trait NonCompositePrimaryKeyTables: Tables<PrimaryKeys: Columns<Tables = Self>> {}

impl<T> NonCompositePrimaryKeyTables for T where T: Tables<PrimaryKeys: Columns<Tables = T>> {}

/// A trait representing a collection of Diesel tables which
/// have homogeneous primary key types.
pub trait CompatiblePrimaryKeys<Type>: Tables<PrimaryKeys: HomogeneousColumns<Type>> {}

impl<T, Type> CompatiblePrimaryKeys<Type> for T where
    T: Tables<PrimaryKeys: HomogeneousColumns<Type>>
{
}

/// Trait representing an n-uple of TableModels.
pub trait TableModels: IntoTupleOption {
    /// The n-uple of tables corresponding to these models.
    type Tables: Tables<Models = Self>;
}

/// Trait representing an n-uple of InsertableTableModels.
pub trait InsertableTableModels: Sized + TupleDefault {
    /// The n-uple of tables corresponding to these insertable models.
    type Tables: Tables<InsertableModels = Self>;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_tables]
mod impls {}
