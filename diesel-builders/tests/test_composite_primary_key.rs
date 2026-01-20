//! Submodule to test whether the diesel-builder can work in the case of
//! a single table with a composite primary key.

mod shared;
use diesel_builders::{
    load_query_builder::{LoadMany, LoadPaginated, LoadSorted},
    prelude::*,
};
use diesel_builders_derive::TableModel;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = user_roles)]
#[diesel(primary_key(user_id, role_id))]
/// A user role assignment model.
pub struct UserRole {
    /// The ID of the user.
    user_id: i32,
    /// The ID of the role.
    role_id: i32,
    /// When the role was assigned.
    assigned_at: String,
}

#[test]
fn test_composite_primary_key_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE user_roles (
			user_id INTEGER NOT NULL,
			role_id INTEGER NOT NULL,
			assigned_at TEXT NOT NULL,
			PRIMARY KEY (user_id, role_id)
		)",
    )
    .execute(&mut conn)?;

    let mut builder = user_roles::table::builder();

    assert_eq!(builder.may_get_column_ref::<user_roles::user_id>(), None);
    assert_eq!(builder.may_get_column_ref::<user_roles::role_id>(), None);
    assert_eq!(builder.may_get_column_ref::<user_roles::assigned_at>(), None);

    builder.try_user_id_ref(1)?;

    assert_eq!(builder.may_get_column_ref::<user_roles::user_id>(), Some(&1));
    assert_eq!(builder.may_get_column_ref::<user_roles::role_id>(), None);
    assert_eq!(builder.may_get_column_ref::<user_roles::assigned_at>(), None);

    builder.try_role_id_ref(10)?;
    builder.try_assigned_at_ref("2025-01-01")?;

    assert_eq!(builder.may_get_column_ref::<user_roles::user_id>(), Some(&1));
    assert_eq!(builder.may_get_column_ref::<user_roles::role_id>(), Some(&10));
    assert_eq!(
        builder.may_get_column_ref::<user_roles::assigned_at>(),
        Some(&"2025-01-01".to_string())
    );

    let builder_clone = builder.clone();
    let user_role = builder.insert(&mut conn)?;

    // We must modify the primary key to avoid unique constraint violation,
    // as the builder still has user_id=1 and role_id=10
    let nested_models = builder_clone.try_role_id(11)?.insert_nested(&mut conn)?;

    // Verify non-PK fields align
    assert_eq!(nested_models.assigned_at(), user_role.assigned_at());
    // Verify part of PK that wasn't changed
    assert_eq!(nested_models.user_id(), user_role.user_id());

    assert_eq!(user_role.user_id(), &1);
    assert_eq!(user_role.role_id(), &10);
    assert_eq!(user_role.assigned_at(), "2025-01-01");

    assert_eq!(user_role.user_id(), &1);
    assert_eq!(user_role.role_id(), &10);
    assert_eq!(user_role.assigned_at(), "2025-01-01");

    // We attempt to query the inserted user role to ensure everything worked
    let queried_user_role: UserRole = UserRole::find(user_role.id(), &mut conn)?;
    assert_eq!(user_role, queried_user_role);

    // We test the chained variant.
    let builder = user_roles::table::builder().user_id(2).role_id(20).assigned_at("2025-02-01");

    let builder_clone = builder.clone();
    let another_user_role = builder.insert(&mut conn)?;

    // We must modify the primary key to avoid unique constraint violation
    let nested_models = builder_clone.role_id(21).insert_nested(&mut conn)?;

    assert_eq!(nested_models.assigned_at(), another_user_role.assigned_at());
    assert_eq!(nested_models.user_id(), another_user_role.user_id());

    assert_eq!(another_user_role.user_id(), &2);
    assert_eq!(another_user_role.role_id(), &20);
    assert_eq!(another_user_role.assigned_at(), "2025-02-01");

    // With composite keys, both user_id and role_id form the primary key,
    // so we expect different combinations to be distinct
    assert_ne!(
        (user_role.user_id(), user_role.role_id()),
        (another_user_role.user_id(), another_user_role.role_id())
    );

    Ok(())
}

#[test]
#[cfg(feature = "serde")]
fn test_builder_serde_serialization() -> Result<(), Box<dyn std::error::Error>> {
    // Create a builder with composite primary key values
    let builder = user_roles::table::builder().user_id(42).role_id(100).assigned_at("2025-12-04");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<user_roles::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(deserialized.may_get_column_ref::<user_roles::user_id>(), Some(&42));
    assert_eq!(deserialized.may_get_column_ref::<user_roles::role_id>(), Some(&100));
    assert_eq!(
        deserialized.may_get_column_ref::<user_roles::assigned_at>().map(String::as_str),
        Some("2025-12-04")
    );

    Ok(())
}

#[test]
fn test_load_many_composite() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE user_roles (
			user_id INTEGER NOT NULL,
			role_id INTEGER NOT NULL,
			assigned_at TEXT NOT NULL,
			PRIMARY KEY (user_id, role_id)
		)",
    )
    .execute(&mut conn)?;

    // Insert data
    let ur1_10 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-01-01")
        .insert(&mut conn)?;

    let ur1_20 = user_roles::table::builder()
        .user_id(1)
        .role_id(20)
        .assigned_at("2025-01-02")
        .insert(&mut conn)?;

    let ur2_10 = user_roles::table::builder()
        .user_id(2)
        .role_id(10)
        .assigned_at("2025-01-03")
        .insert(&mut conn)?;

    // Test LoadMany
    let roles_user_1: Vec<UserRole> = <(user_roles::user_id,)>::load_many((1,), &mut conn)?;
    assert_eq!(roles_user_1.len(), 2);
    assert!(roles_user_1.contains(&ur1_10));
    assert!(roles_user_1.contains(&ur1_20));

    let users_role_10: Vec<UserRole> = <(user_roles::role_id,)>::load_many((10,), &mut conn)?;
    assert_eq!(users_role_10.len(), 2);
    assert!(users_role_10.contains(&ur1_10));
    assert!(users_role_10.contains(&ur2_10));

    // Test LoadSorted
    // Sorted by PK (user_id, role_id)
    let roles_user_1_sorted: Vec<UserRole> =
        <(user_roles::user_id,)>::load_sorted((1,), &mut conn)?;
    assert_eq!(roles_user_1_sorted, vec![ur1_10.clone(), ur1_20.clone()]);

    // Test LoadPaginated
    // Get first item only
    let roles_user_1_paginated: Vec<UserRole> =
        <(user_roles::user_id,)>::load_many_paginated((1,), 0, 1, &mut conn)?;
    assert_eq!(roles_user_1_paginated, vec![ur1_10.clone()]);

    // Get second item using offset
    let roles_user_1_offset: Vec<UserRole> =
        <(user_roles::user_id,)>::load_many_paginated((1,), 1, 1, &mut conn)?;
    assert_eq!(roles_user_1_offset, vec![ur1_20.clone()]);

    // Get all items with high limit
    let roles_user_1_all: Vec<UserRole> =
        <(user_roles::user_id,)>::load_many_paginated((1,), 0, 10, &mut conn)?;
    assert_eq!(roles_user_1_all, vec![ur1_10, ur1_20]);

    Ok(())
}

#[test]
fn test_upsert_composite() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE user_roles (
			user_id INTEGER NOT NULL,
			role_id INTEGER NOT NULL,
			assigned_at TEXT NOT NULL,
			PRIMARY KEY (user_id, role_id)
		)",
    )
    .execute(&mut conn)?;

    // 1. Insert initial record
    let mut user_role = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-01-01")
        .insert(&mut conn)?;

    assert_eq!(user_role.assigned_at, "2025-01-01");

    // 2. Upsert (Update)
    user_role.assigned_at = "2025-01-02".to_string();

    let updated_role = user_role.upsert(&mut conn)?;
    assert_eq!(updated_role.assigned_at, "2025-01-02");
    assert_eq!(updated_role.user_id, 1);
    assert_eq!(updated_role.role_id, 10);

    // Verify in DB
    let queried_role: UserRole = UserRole::find(updated_role.id(), &mut conn)?;
    assert_eq!(queried_role.assigned_at, "2025-01-02");

    // 3. Upsert (Insert)
    // We need to construct a UserRole manually since we don't have a builder that
    // returns a struct without inserting. But we can use the struct constructor
    // since fields are public now. TODO: We will add support for upsert via
    // builder in the future.
    let new_role = UserRole { user_id: 2, role_id: 20, assigned_at: "2025-02-01".to_string() };

    let inserted_role = new_role.upsert(&mut conn)?;
    assert_eq!(inserted_role.user_id, 2);
    assert_eq!(inserted_role.role_id, 20);
    assert_eq!(inserted_role.assigned_at, "2025-02-01");

    // Verify in DB
    let queried_new_role: UserRole = UserRole::find(inserted_role.id(), &mut conn)?;
    assert_eq!(queried_new_role, inserted_role);

    Ok(())
}
