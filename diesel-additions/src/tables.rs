//! Submodule defining and implementing the `Tables` trait.

use diesel::associations::HasTable;

use crate::{Columns, DefaultTuple, InsertableTableModel, OptionTuple, TableAddition, TableModel};

/// A trait representing a collection of Diesel tables.
pub trait Tables {
    /// The n-uple of models corresponding to the tables in this collection.
    type Models: TableModels;
    /// The n-uple of insertable models corresponding to the tables in this
    /// collection.
    type InsertableModels: InsertableTableModels;
}

/// A trait representing a collection of Diesel tables which
/// have non-composite primary keys.
pub trait NonCompositePrimaryKeyTables: Tables {
    /// The tuple of non-composite primary key columns for these tables.
    type PrimaryKeys: Columns<Tables = Self>;
}

/// Trait representing an n-uple of TableModels.
pub trait TableModels: OptionTuple {
    /// The n-uple of tables corresponding to these models.
    type Tables: Tables<Models = Self>;
}

/// Trait representing an n-uple of InsertableTableModels.
pub trait InsertableTableModels: Sized + DefaultTuple {
    /// The n-uple of tables corresponding to these insertable models.
    type Tables: Tables<InsertableModels = Self>;
}

// Generate implementations for all tuple sizes (0-32)
#[diesel_builders_macros::impl_tables]
mod impls {}
