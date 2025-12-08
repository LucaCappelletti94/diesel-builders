//! Test for TableModel derive with custom insertable override.

mod common;

use diesel::prelude::*;
use diesel_builders::{TableAddition, prelude::*};

diesel::table! {
    /// Override table for testing.
    override_table (id) {
        /// ID column.
        id -> Integer,
    }
}

/// Model for override table.
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, Root, TableModel)]
#[table_model(insertable = MyInsertable)]
#[diesel(table_name = override_table)]
pub struct OverrideModel {
    /// ID field.
    pub id: i32,
}

/// Custom insertable model.
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = override_table)]
pub struct MyInsertable {
    /// ID field.
    pub id: Option<i32>,
}

#[test]
fn test_table_model_insertable_override() {
    // If the derive worked, the associated InsertableModel must be MyInsertable.
    let _v: <override_table::table as TableAddition>::InsertableModel = MyInsertable::default();
}
