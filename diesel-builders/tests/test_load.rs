//! Test case for `LoadMany` and `LoadManySorted` traits.

mod shared;
use diesel::prelude::*;
use diesel_builders::{
    load_query_builder::{LoadMany, LoadManySorted, LoadManySortedPaginated},
    prelude::*,
};
use diesel_builders_derive::TableModel;

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
    let item1 = items::table::builder().category(1).val(10).insert(&mut conn)?;
    let item2 = items::table::builder().category(1).val(20).insert(&mut conn)?;
    let _item3 = items::table::builder().category(2).val(30).insert(&mut conn)?;

    // Test LoadMany
    let loaded_items: Vec<Item> = <(items::category,)>::load_many((1,), &mut conn)?;

    assert_eq!(loaded_items.len(), 2);
    assert!(loaded_items.contains(&item1));
    assert!(loaded_items.contains(&item2));

    let loaded_items2: Vec<Item> =
        <(items::category, (items::val,))>::load_many((1, (20,)), &mut conn)?;
    assert_eq!(loaded_items2, vec![item2]);

    Ok(())
}

#[test]
fn test_load_many_sorted() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    // Insert Items
    let item1 = items::table::builder().category(1).val(10).insert(&mut conn)?;

    let item2 = items::table::builder().category(1).val(20).insert(&mut conn)?;

    let _item3 = items::table::builder().category(2).val(30).insert(&mut conn)?;

    // Test LoadManySorted
    // Should be sorted by Primary Key (id)
    let sorted_items: Vec<Item> = <(items::category,)>::load_many_sorted((1,), &mut conn)?;

    assert_eq!(sorted_items, vec![item1, item2.clone()]);

    let sorted_items2: Vec<Item> =
        <(items::category, (items::val,))>::load_many_sorted((1, (20,)), &mut conn)?;

    assert_eq!(sorted_items2, vec![item2]);

    Ok(())
}

#[test]
fn test_load_many_sorted_paginated() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    create_tables(&mut conn)?;

    // Insert Items
    let item1 = items::table::builder().category(1).val(10).insert(&mut conn)?;

    let item2 = items::table::builder().category(1).val(20).insert(&mut conn)?;

    let item3 = items::table::builder().category(1).val(30).insert(&mut conn)?;

    let _item4 = items::table::builder().category(2).val(40).insert(&mut conn)?;

    // Test LoadManySortedPaginated - get all with limit
    let paginated_items: Vec<Item> =
        <(items::category,)>::load_many_sorted_paginated((1,), 0, 2, &mut conn)?;

    assert_eq!(paginated_items, vec![item1, item2.clone()]);

    // Test with offset - skip first item
    let paginated_items_offset: Vec<Item> =
        <(items::category,)>::load_many_sorted_paginated((1,), 1, 2, &mut conn)?;

    assert_eq!(paginated_items_offset, vec![item2.clone(), item3]);

    // Test with offset and limit - get only middle item
    let paginated_items_middle: Vec<Item> =
        <(items::category,)>::load_many_sorted_paginated((1,), 1, 1, &mut conn)?;

    assert_eq!(paginated_items_middle, vec![item2.clone()]);

    // Test with additional filter column
    let paginated_items_filtered: Vec<Item> =
        <(items::category, (items::val,))>::load_many_sorted_paginated(
            (1, (20,)),
            0,
            10,
            &mut conn,
        )?;

    assert_eq!(paginated_items_filtered, vec![item2]);

    Ok(())
}
