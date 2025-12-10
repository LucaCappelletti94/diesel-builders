//! Test verifying the correct functioning of the `TryFrom` between `RecursiveTableBuilder` and `TableBuilder`
use std::convert::Infallible;

use diesel_builders::{
    IncompleteBuilderError, TableBuilder, prelude::*, table_builder::RecursiveTableBuilder,
};
use typenum::U0;

diesel::table! {
    /// Dogs table - extends animals via foreign key.
    dogs (id) {
        /// Primary key of the dog, foreign key to animals.id.
        id -> Integer,
        /// The breed of the dog.
        breed -> Text,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel, Root,
)]
#[diesel(table_name = dogs)]
/// Model struct associated to the `dogs` table.
pub struct Dog {
    id: i32,
    breed: String,
}

#[derive(
    Debug, Clone, Default, diesel::Selectable, diesel::Insertable, PartialEq, PartialOrd, HasTable,
)]
#[diesel(table_name = dogs)]
/// Model struct associated to the `dogs` table.
pub struct NewDog {
    id: Option<i32>,
    breed: Option<String>,
}

impl InsertableTableModel for NewDog {
    type Error = Infallible;
}

#[test]
fn test_try_from_table_builder() {
    // Add test code here
    let builder: TableBuilder<dogs::table> = dogs::table::builder();

    type Target = RecursiveTableBuilder<
        dogs::table,
        U0,
        <dogs::table as BuildableTable>::NestedCompletedAncestorBuilders,
    >;

    let _recursive_builder: Result<Target, IncompleteBuilderError> = Target::try_from(builder);
}
