//! Example: Simple Table (Base Case)
//!
//! This example demonstrates a single table with no relationships and includes
//! validation using custom TrySetColumn implementations.
//!
//! Run with: `cargo run --example simple_table`

use diesel_builders::prelude::*;

diesel::table! {
    /// Users table schema
    users (id) {
        /// User ID (primary key)
        id -> Integer,
        /// User's display name
        name -> Text,
        /// User's email address
        email -> Text,
        /// Optional user biography
        bio -> Nullable<Text>,
    }
}

/// User model representing a row in the users table
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, Root, TableModel)]
#[diesel(table_name = users)]
pub struct User {
    /// User ID (primary key)
    pub id: i32,
    /// User's display name
    pub name: String,
    /// User's email address
    pub email: String,
    /// Optional user biography
    pub bio: Option<String>,
}

/// Builder for creating new users with validation
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, HasTable)]
#[diesel(table_name = users)]
pub struct NewUser {
    /// User's display name
    pub name: Option<String>,
    /// User's email address
    pub email: Option<String>,
    /// Optional user biography
    pub bio: Option<Option<String>>,
}

// Custom validation for name column - must be non-empty
impl diesel_builders::TrySetColumn<users::name> for NewUser {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("Name cannot be empty or whitespace-only");
        }
        if value.len() > 100 {
            anyhow::bail!("Name cannot exceed 100 characters");
        }
        self.name = Some(value.clone());
        Ok(self)
    }
}

// Custom validation for email column - must be non-empty
impl diesel_builders::TrySetColumn<users::email> for NewUser {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("Email cannot be empty");
        }
        self.email = Some(value.clone());
        Ok(self)
    }
}

// Bio is optional and can use default implementation via SetColumn derive
// But we'll implement TrySetColumn for length validation
impl diesel_builders::TrySetColumn<users::bio> for NewUser {
    fn try_set_column(&mut self, value: &Option<String>) -> anyhow::Result<&mut Self> {
        if let Some(bio) = value
            && bio.len() > 500
        {
            anyhow::bail!("Bio cannot exceed 500 characters");
        }
        self.bio = Some(value.clone());
        Ok(self)
    }
}

impl TableAddition for users::table {
    type InsertableModel = NewUser;
    type Model = User;
    type InsertableColumns = (users::name, users::email, users::bio);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Create the table with CHECK constraints
    diesel::sql_query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 100),
            email TEXT NOT NULL CHECK(length(trim(email)) > 0),
            bio TEXT CHECK(bio IS NULL OR length(bio) <= 500)
        )",
    )
    .execute(&mut conn)?;

    // Use the builder to insert a valid user
    let user: User = users::table::builder()
        .try_set_column::<users::name>(&"Alice".to_string())?
        .try_set_column::<users::email>(&"alice@example.com".to_string())?
        .insert(&mut conn)?;

    println!("Successfully inserted user: {user:?}");
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.bio, None);

    // Test validation - empty name should fail
    let mut builder = users::table::builder();
    let result = builder.try_set_column_ref::<users::name>(&"   ".to_string());
    assert!(result.is_err());
    println!("Empty name validation works: {}", result.unwrap_err());

    // Test validation - empty email should fail
    let mut builder = users::table::builder();
    builder.try_set_column_ref::<users::name>(&"Bob".to_string())?;
    let result = builder.try_set_column_ref::<users::email>(&"   ".to_string());
    assert!(result.is_err());
    println!("Email validation works: {}", result.unwrap_err());

    // Test validation - bio too long should fail
    let long_bio = "x".repeat(501);
    let mut builder = users::table::builder();
    builder.try_set_column_ref::<users::name>(&"Charlie".to_string())?;
    builder.try_set_column_ref::<users::email>(&"charlie@example.com".to_string())?;
    let result = builder.try_set_column_ref::<users::bio>(&Some(long_bio));
    assert!(result.is_err());
    println!("Bio length validation works: {}", result.unwrap_err());

    println!("All validations passed!");
    Ok(())
}
