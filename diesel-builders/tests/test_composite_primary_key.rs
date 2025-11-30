//! Submodule to test whether the diesel-builder can work in the case of
//! a single table with a composite primary key.

mod common;

use diesel::prelude::*;
use diesel_additions::{
    GetColumnExt, MayGetColumnExt, SetColumnExt, TableAddition, TrySetColumnExt,
};
use diesel_builders::{BuildableTable, BundlableTable, NestedInsert};
use diesel_builders_macros::{
    GetColumn, HasTable, MayGetColumn, NoHorizontalSameAsGroup, Root, SetColumn, TableModel,
};

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
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    GetColumn,
    Root,
    TableModel,
    NoHorizontalSameAsGroup,
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

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
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

impl TableAddition for user_roles::table {
    type InsertableModel = NewUserRole;
    type Model = UserRole;
    type InsertableColumns = (user_roles::user_id, user_roles::role_id, user_roles::assigned_at);
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

    assert_eq!(builder.may_get_column::<user_roles::user_id>(), None);
    assert_eq!(builder.may_get_column::<user_roles::role_id>(), None);
    assert_eq!(builder.may_get_column::<user_roles::assigned_at>(), None);

    builder.try_set_column::<user_roles::user_id>(&1)?;

    assert_eq!(builder.may_get_column::<user_roles::user_id>(), Some(&1));
    assert_eq!(builder.may_get_column::<user_roles::role_id>(), None);
    assert_eq!(builder.may_get_column::<user_roles::assigned_at>(), None);

    builder.try_set_column::<user_roles::role_id>(&10)?;
    builder.try_set_column::<user_roles::assigned_at>(&"2025-01-01".to_string())?;

    assert_eq!(builder.may_get_column::<user_roles::user_id>(), Some(&1));
    assert_eq!(builder.may_get_column::<user_roles::role_id>(), Some(&10));
    assert_eq!(
        builder.may_get_column::<user_roles::assigned_at>(),
        Some(&"2025-01-01".to_string())
    );

    let user_role = builder.insert(&mut conn)?;

    assert_eq!(user_role.user_id, 1);
    assert_eq!(user_role.role_id, 10);
    assert_eq!(user_role.assigned_at, "2025-01-01");

    assert_eq!(user_role.get_column::<user_roles::user_id>(), &1);
    assert_eq!(user_role.get_column::<user_roles::role_id>(), &10);
    assert_eq!(user_role.get_column::<user_roles::assigned_at>(), &"2025-01-01".to_string());

    // We attempt to query the inserted user role to ensure everything worked
    let queried_user_role: UserRole = user_roles::table
        .filter(user_roles::user_id.eq(user_role.user_id))
        .filter(user_roles::role_id.eq(user_role.role_id))
        .first(&mut conn)?;
    assert_eq!(user_role, queried_user_role);

    // We test the chained variant.
    let another_user_role = user_roles::table::builder()
        .set_column::<user_roles::user_id>(&2)
        .set_column::<user_roles::role_id>(&20)
        .set_column::<user_roles::assigned_at>(&"2025-02-01".to_string())
        .insert(&mut conn)?;

    assert_eq!(another_user_role.get_column::<user_roles::user_id>(), &2);
    assert_eq!(another_user_role.get_column::<user_roles::role_id>(), &20);
    assert_eq!(
        another_user_role.get_column::<user_roles::assigned_at>(),
        &"2025-02-01".to_string()
    );

    // With composite keys, both user_id and role_id form the primary key,
    // so we expect different combinations to be distinct
    assert_ne!(
        (
            user_role.get_column::<user_roles::user_id>(),
            user_role.get_column::<user_roles::role_id>()
        ),
        (
            another_user_role.get_column::<user_roles::user_id>(),
            another_user_role.get_column::<user_roles::role_id>()
        )
    );

    Ok(())
}
