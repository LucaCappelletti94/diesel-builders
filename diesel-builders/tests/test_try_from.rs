//! Test verifying the correct functioning of the `TryFrom` between `RecursiveTableBuilder` and `TableBuilder`
use diesel_builders::{
    IncompleteBuilderError, TableBuilder, prelude::*, table_builder::RecursiveTableBuilder,
};
use typenum::U0;

#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = dogs)]
/// Model struct associated to the `dogs` table.
pub struct Dog {
    /// The ID of the dog.
    id: i32,
    /// The breed of the dog.
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
