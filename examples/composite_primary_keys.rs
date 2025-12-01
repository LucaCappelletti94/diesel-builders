//! Example: Composite Primary Keys
//!
//! This example demonstrates tables with multi-column primary keys and validation.
//!
//! Run with: `cargo run --example composite_primary_keys`

use diesel_builders::prelude::*;

diesel::table! {
    /// User roles table schema with composite primary key
    user_roles (user_id, role_id) {
        /// User ID (part of composite primary key)
        user_id -> Integer,
        /// Role ID (part of composite primary key)
        role_id -> Integer,
        /// Description or note about the role assignment
        assigned_at -> Text,
    }
}

/// User role model representing a row in the user_roles table
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, Root, TableModel)]
#[diesel(table_name = user_roles)]
#[diesel(primary_key(user_id, role_id))]
pub struct UserRole {
    /// User ID (part of composite primary key)
    pub user_id: i32,
    /// Role ID (part of composite primary key)
    pub role_id: i32,
    /// Description or note about the role assignment
    pub assigned_at: String,
}

/// Builder for creating new user role assignments with validation
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, HasTable)]
#[diesel(table_name = user_roles)]
pub struct NewUserRole {
    /// User ID (part of composite primary key)
    pub user_id: Option<i32>,
    /// Role ID (part of composite primary key)
    pub role_id: Option<i32>,
    /// Date when role was assigned (ISO 8601 format)
    pub assigned_at: Option<String>,
}

// Validation for user_id - must be positive
impl diesel_builders::TrySetColumn<user_roles::user_id> for NewUserRole {
    fn try_set_column(&mut self, value: &i32) -> anyhow::Result<&mut Self> {
        if *value <= 0 {
            anyhow::bail!("User ID must be a positive integer");
        }
        self.user_id = Some(*value);
        Ok(self)
    }
}

// Validation for role_id - must be positive and within valid range
impl diesel_builders::TrySetColumn<user_roles::role_id> for NewUserRole {
    fn try_set_column(&mut self, value: &i32) -> anyhow::Result<&mut Self> {
        if *value <= 0 {
            anyhow::bail!("Role ID must be a positive integer");
        }
        if *value > 1000 {
            anyhow::bail!("Role ID must be between 1 and 1000");
        }
        self.role_id = Some(*value);
        Ok(self)
    }
}

// Validation for assigned_at - must be non-empty
impl diesel_builders::TrySetColumn<user_roles::assigned_at> for NewUserRole {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("Assignment note cannot be empty");
        }
        self.assigned_at = Some(value.clone());
        Ok(self)
    }
}

impl TableAddition for user_roles::table {
    type InsertableModel = NewUserRole;
    type Model = UserRole;
    type InsertableColumns = (
        user_roles::user_id,
        user_roles::role_id,
        user_roles::assigned_at,
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Create table with composite primary key and CHECK constraints
    diesel::sql_query(
        "CREATE TABLE user_roles (
            user_id INTEGER NOT NULL CHECK(user_id > 0),
            role_id INTEGER NOT NULL CHECK(role_id > 0 AND role_id <= 1000),
            assigned_at TEXT NOT NULL CHECK(length(trim(assigned_at)) > 0),
            PRIMARY KEY (user_id, role_id)
        )",
    )
    .execute(&mut conn)?;

    // Use builder to insert a valid record
    let user_role: UserRole = user_roles::table::builder()
        .try_set_column::<user_roles::user_id>(&42)?
        .try_set_column::<user_roles::role_id>(&10)?
        .try_set_column::<user_roles::assigned_at>(&"admin role".to_string())?
        .insert(&mut conn)?;

    println!("Successfully inserted user_role: {user_role:?}");
    assert_eq!(user_role.user_id, 42);
    assert_eq!(user_role.role_id, 10);
    assert_eq!(user_role.assigned_at, "admin role");

    // Test validation - invalid user_id
    let mut builder = user_roles::table::builder();
    let result = builder.try_set_column::<user_roles::user_id>(&-1);
    assert!(result.is_err());
    println!("User ID validation works: {}", result.unwrap_err());

    // Test validation - role_id out of range
    let mut builder = user_roles::table::builder();
    builder.try_set_column::<user_roles::user_id>(&1)?;
    let result = builder.try_set_column::<user_roles::role_id>(&1001);
    assert!(result.is_err());
    println!("Role ID range validation works: {}", result.unwrap_err());

    // Test validation - empty assignment note
    let mut builder = user_roles::table::builder();
    builder.try_set_column::<user_roles::user_id>(&1)?;
    builder.try_set_column::<user_roles::role_id>(&5)?;
    let result = builder.try_set_column::<user_roles::assigned_at>(&"   ".to_string());
    assert!(result.is_err());
    println!("Assignment note validation works: {}", result.unwrap_err());

    println!("All validations passed!");
    Ok(())
}
