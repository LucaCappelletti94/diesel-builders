//! Submodule to test whether the diesel-builder can work in the case of
//! a single table with a composite primary key.

mod common;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel};
use std::collections::HashMap;

diesel::table! {
    /// Define a user_roles table with composite primary key for testing.
    user_roles (user_id, role_id) {
        /// The ID of the user.
        user_id -> Integer,
        /// The ID of the role.
        role_id -> Integer,
        /// Additional data about the assignment.
        assigned_at -> Text,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = user_roles)]
#[diesel(primary_key(user_id, role_id))]
/// A user role assignment model.
pub struct UserRole {
    /// The ID of the user.
    pub user_id: i32,
    /// The ID of the role.
    pub role_id: i32,
    /// When the role was assigned.
    pub assigned_at: String,
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Insertable,
    MayGetColumn,
    SetColumn,
    HasTable,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = user_roles)]
/// A new user role model for insertions.
pub struct NewUserRole {
    /// The ID of the user.
    pub user_id: Option<i32>,
    /// The ID of the role.
    pub role_id: Option<i32>,
    /// When the role was assigned.
    pub assigned_at: Option<String>,
}

#[test]
fn test_composite_primary_key_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

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

    assert_eq!(user_role.user_id, 1);
    assert_eq!(user_role.role_id, 10);
    assert_eq!(user_role.assigned_at, "2025-01-01");

    assert_eq!(user_role.user_id(), &1);
    assert_eq!(user_role.role_id(), &10);
    assert_eq!(user_role.assigned_at(), "2025-01-01");

    // We attempt to query the inserted user role to ensure everything worked
    let queried_user_role: UserRole = user_roles::table
        .filter(user_roles::user_id.eq(user_role.user_id))
        .filter(user_roles::role_id.eq(user_role.role_id))
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
fn test_composite_primary_key_builder_equality() {
    // Test PartialEq for composite primary key builders
    let builder1 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-01-01");

    let builder2 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-01-01");

    let builder3 = user_roles::table::builder()
        .user_id(2)
        .role_id(10)
        .assigned_at("2025-01-01");

    let builder4 = user_roles::table::builder()
        .user_id(1)
        .role_id(20)
        .assigned_at("2025-01-01");

    let builder5 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-02-01");

    // Identical builders should be equal
    assert_eq!(builder1, builder2);

    // Different builders should not be equal
    assert_ne!(builder1, builder3);
    assert_ne!(builder1, builder4);
    assert_ne!(builder1, builder5);
    assert_ne!(builder3, builder4);
    assert_ne!(builder3, builder5);
    assert_ne!(builder4, builder5);

    // The builders should also be equal to themselves
    assert_eq!(builder1, builder1);
    assert_eq!(builder2, builder2);
    assert_eq!(builder3, builder3);
    assert_eq!(builder4, builder4);
    assert_eq!(builder5, builder5);
}

#[test]
fn test_composite_primary_key_builder_hash() {
    // Test Hash for composite primary key builders
    let builder1 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-01-01");

    let builder2 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-01-01");

    let builder3 = user_roles::table::builder()
        .user_id(2)
        .role_id(10)
        .assigned_at("2025-01-01");

    let builder4 = user_roles::table::builder()
        .user_id(1)
        .role_id(20)
        .assigned_at("2025-01-01");

    let builder5 = user_roles::table::builder()
        .user_id(1)
        .role_id(10)
        .assigned_at("2025-02-01");

    // Calculate hashes
    let mut hasher1 = DefaultHasher::new();
    builder1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    builder2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    let mut hasher3 = DefaultHasher::new();
    builder3.hash(&mut hasher3);
    let hash3 = hasher3.finish();

    let mut hasher4 = DefaultHasher::new();
    builder4.hash(&mut hasher4);
    let hash4 = hasher4.finish();

    let mut hasher5 = DefaultHasher::new();
    builder5.hash(&mut hasher5);
    let hash5 = hasher5.finish();

    // Identical builders should have the same hash
    assert_eq!(hash1, hash2);

    // Different builders should have different hashes
    assert_ne!(hash1, hash3);
    assert_ne!(hash1, hash4);
    assert_ne!(hash1, hash5);
    assert_ne!(hash3, hash4);
    assert_ne!(hash3, hash5);
    assert_ne!(hash4, hash5);

    // Test that builders can be used as HashMap keys
    let mut map = HashMap::new();
    map.insert(builder1.clone(), "user1_admin");
    map.insert(builder3.clone(), "user2_admin");
    map.insert(builder4.clone(), "user1_moderator");

    assert_eq!(map.get(&builder2), Some(&"user1_admin"));
    assert_eq!(map.get(&builder5), None);
    assert_eq!(map.get(&builder3), Some(&"user2_admin"));
    assert_eq!(map.get(&builder4), Some(&"user1_moderator"));
}

#[test]
fn test_composite_primary_key_builder_partial_ord() {
    // Test PartialOrd implementation for TableBuilder
    let builder1 = user_roles::table::builder()
        .user_id(1)
        .role_id(1)
        .assigned_at("2025-01-01");

    let builder2 = user_roles::table::builder()
        .user_id(1)
        .role_id(1)
        .assigned_at("2025-01-01");

    let builder3 = user_roles::table::builder()
        .user_id(2)
        .role_id(1)
        .assigned_at("2025-01-01");

    let builder4 = user_roles::table::builder()
        .user_id(1)
        .role_id(2)
        .assigned_at("2025-01-01");

    let builder5 = user_roles::table::builder()
        .user_id(1)
        .role_id(1)
        .assigned_at("2025-01-02");

    // Identical builders should be equal
    assert_eq!(
        builder1.partial_cmp(&builder2),
        Some(std::cmp::Ordering::Equal)
    );

    // Different builders should have proper ordering
    assert_eq!(
        builder1.partial_cmp(&builder3),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        builder3.partial_cmp(&builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        builder1.partial_cmp(&builder4),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        builder4.partial_cmp(&builder1),
        Some(std::cmp::Ordering::Greater)
    );
    assert_eq!(
        builder1.partial_cmp(&builder5),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        builder5.partial_cmp(&builder1),
        Some(std::cmp::Ordering::Greater)
    );

    // Test Ord implementation
    assert_eq!(builder1.cmp(&builder2), std::cmp::Ordering::Equal);
    assert_eq!(builder1.cmp(&builder3), std::cmp::Ordering::Less);
    assert_eq!(builder3.cmp(&builder1), std::cmp::Ordering::Greater);
    assert_eq!(builder1.cmp(&builder4), std::cmp::Ordering::Less);
    assert_eq!(builder4.cmp(&builder1), std::cmp::Ordering::Greater);
    assert_eq!(builder1.cmp(&builder5), std::cmp::Ordering::Less);
    assert_eq!(builder5.cmp(&builder1), std::cmp::Ordering::Greater);
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
