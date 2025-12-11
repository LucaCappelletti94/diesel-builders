//! Test verifying the correct functioning of the `TryFrom` between `RecursiveTableBuilder` and `TableBuilder`
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

#[test]
fn test_try_from_table_builder() {
    // Add test code here
    type Target = RecursiveTableBuilder<
        dogs::table,
        U0,
        <dogs::table as BuildableTable>::NestedCompletedAncestorBuilders,
    >;

    let builder: TableBuilder<dogs::table> = dogs::table::builder();

    let _recursive_builder: Result<Target, IncompleteBuilderError> = Target::try_from(builder);
}
