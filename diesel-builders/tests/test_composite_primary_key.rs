//! Submodule to test whether the diesel-builder can work in the case of
//! a single table with a composite primary key.

mod shared;
use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::{Root, TableModel};

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, Root, TableModel)]
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
    assert_eq!(
        builder.may_get_column_ref::<user_roles::assigned_at>(),
        None
    );

    builder.try_user_id_ref(1)?;

    assert_eq!(
        builder.may_get_column_ref::<user_roles::user_id>(),
        Some(&1)
    );
    assert_eq!(builder.may_get_column_ref::<user_roles::role_id>(), None);
    assert_eq!(
        builder.may_get_column_ref::<user_roles::assigned_at>(),
        None
    );

    builder.try_role_id_ref(10)?;
    builder.try_assigned_at_ref("2025-01-01")?;

    assert_eq!(
        builder.may_get_column_ref::<user_roles::user_id>(),
        Some(&1)
    );
    assert_eq!(
        builder.may_get_column_ref::<user_roles::role_id>(),
        Some(&10)
    );
    assert_eq!(
        builder.may_get_column_ref::<user_roles::assigned_at>(),
        Some(&"2025-01-01".to_string())
    );

    let user_role = builder.insert(&mut conn)?;

    assert_eq!(user_role.user_id(), &1);
    assert_eq!(user_role.role_id(), &10);
    assert_eq!(user_role.assigned_at(), "2025-01-01");

    assert_eq!(user_role.user_id(), &1);
    assert_eq!(user_role.role_id(), &10);
    assert_eq!(user_role.assigned_at(), "2025-01-01");

    // We attempt to query the inserted user role to ensure everything worked
    let queried_user_role: UserRole = user_roles::table
        .filter(user_roles::user_id.eq(user_role.user_id()))
        .filter(user_roles::role_id.eq(user_role.role_id()))
        .first(&mut conn)?;
    assert_eq!(user_role, queried_user_role);

    // We test the chained variant.
    let another_user_role = user_roles::table::builder()
        .user_id(2)
        .role_id(20)
        .assigned_at("2025-02-01")
        .insert(&mut conn)?;

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
    let builder = user_roles::table::builder()
        .user_id(42)
        .role_id(100)
        .assigned_at("2025-12-04");

    // Serialize to JSON
    let serialized = serde_json::to_string(&builder)?;

    // Deserialize back from JSON
    let deserialized: diesel_builders::TableBuilder<user_roles::table> =
        serde_json::from_str(&serialized)?;

    // Verify the values match
    assert_eq!(
        deserialized.may_get_column_ref::<user_roles::user_id>(),
        Some(&42)
    );
    assert_eq!(
        deserialized.may_get_column_ref::<user_roles::role_id>(),
        Some(&100)
    );
    assert_eq!(
        deserialized
            .may_get_column_ref::<user_roles::assigned_at>()
            .map(String::as_str),
        Some("2025-12-04")
    );

    Ok(())
}
