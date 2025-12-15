//! Test case for `LoadMany` and `LoadManySorted` traits.

mod shared;
use diesel::prelude::*;
use diesel_builders::load_query_builder::{LoadMany, LoadManySorted};
use diesel_builders::prelude::*;
use diesel_builders_macros::TableModel;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = items)]
#[table_model(surrogate_key)]
/// Model for items table.
pub struct Item {
    /// Primary key.
    id: i32,
    /// Category column.
    category: i32,
    /// Value column.
    val: i32,
}

fn create_tables(conn: &mut SqliteConnection) -> Result<(), Box<dyn std::error::Error>> {
    diesel::sql_query(
        "CREATE TABLE items (id INTEGER PRIMARY KEY NOT NULL, category INTEGER NOT NULL, val INTEGER NOT NULL)"
    )
    .execute(conn)?;
    Ok(())
}

#[test]
fn test_load_many() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    // Insert Items
    let item1 = items::table::builder()
        .category(1)
        .val(10)
        .insert(&mut conn)?;
    let item2 = items::table::builder()
        .category(1)
        .val(20)
        .insert(&mut conn)?;
    let _item3 = items::table::builder()
        .category(2)
        .val(30)
        .insert(&mut conn)?;

    // Test LoadMany
    let loaded_items: Vec<Item> = <(items::category,)>::load_many((1,), &mut conn)?;

    assert_eq!(loaded_items.len(), 2);
    assert!(loaded_items.contains(&item1));
    assert!(loaded_items.contains(&item2));

    Ok(())
}

#[test]
fn test_load_many_sorted() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    // Insert Items
    let item1 = items::table::builder()
        .category(1)
        .val(10)
        .insert(&mut conn)?;

    let item2 = items::table::builder()
        .category(1)
        .val(20)
        .insert(&mut conn)?;

    let _item3 = items::table::builder()
        .category(2)
        .val(30)
        .insert(&mut conn)?;

    // Test LoadManySorted
    // Should be sorted by Primary Key (id)
    let sorted_items: Vec<Item> = <(items::category,)>::load_many_sorted((1,), &mut conn)?;

    assert_eq!(sorted_items, vec![item1, item2]);

    Ok(())
}
