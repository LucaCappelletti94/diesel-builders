//! Example: Table Inheritance
//!
//! This example demonstrates tables extending a parent table via foreign key
//! on the primary key, with custom validation.
//!
//! Run with: `cargo run --example table_inheritance`

use diesel_builders::prelude::*;

diesel::table! {
    /// Users table schema (parent table)
    users (id) {
        /// User ID (primary key)
        id -> Integer,
        /// User's name
        name -> Text,
        /// User's email address
        email -> Text,
    }
}

diesel::table! {
    /// User profiles table schema (inherits from users)
    user_profiles (id) {
        /// Profile ID (primary key, foreign key to users.id)
        id -> Integer,
        /// User's biography
        bio -> Text,
        /// User's avatar URL
        avatar_url -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(users, user_profiles);

/// User model representing a row in the users table
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, Root, TableModel)]
#[diesel(table_name = users)]
pub struct User {
    /// User ID (primary key)
    pub id: i32,
    /// User's name
    pub name: String,
    /// User's email address
    pub email: String,
}

/// Builder for creating new users with validation
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, HasTable)]
#[diesel(table_name = users)]
pub struct NewUser {
    /// User's name
    pub name: Option<String>,
    /// User's email address
    pub email: Option<String>,
}

// Validation for user name
impl diesel_builders::TrySetColumn<users::name> for NewUser {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("User name cannot be empty");
        }
        if value.len() > 100 {
            anyhow::bail!("User name cannot exceed 100 characters");
        }
        self.name = Some(value.clone());
        Ok(self)
    }
}

// Validation for user email
impl diesel_builders::TrySetColumn<users::email> for NewUser {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("Email cannot be empty");
        }
        self.email = Some(value.clone());
        Ok(self)
    }
}

impl TableAddition for users::table {
    type InsertableModel = NewUser;
    type Model = User;
    type InsertableColumns = (users::name, users::email);
}

/// User profile model representing a row in the user_profiles table
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, TableModel, Decoupled)]
#[diesel(table_name = user_profiles)]
pub struct UserProfile {
    /// Profile ID (primary key, foreign key to users.id)
    pub id: i32,
    /// User's biography
    pub bio: String,
    /// User's avatar URL
    pub avatar_url: String,
}

#[descendant_of]
impl Descendant for user_profiles::table {
    type Ancestors = (users::table,);
    type Root = users::table;
}

/// Builder for creating new user profiles with validation
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, HasTable)]
#[diesel(table_name = user_profiles)]
pub struct NewUserProfile {
    /// Profile ID (will be set to match parent user ID)
    pub id: Option<i32>,
    /// User's biography
    pub bio: Option<String>,
    /// User's avatar URL
    pub avatar_url: Option<String>,
}

// ID is set automatically by the builder from parent, but we need the trait impl
impl diesel_builders::TrySetColumn<user_profiles::id> for NewUserProfile {
    fn try_set_column(&mut self, value: &i32) -> anyhow::Result<&mut Self> {
        self.id = Some(*value);
        Ok(self)
    }
}

// Validation for bio
impl diesel_builders::TrySetColumn<user_profiles::bio> for NewUserProfile {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("Bio cannot be empty or whitespace-only");
        }
        if value.len() > 1000 {
            anyhow::bail!("Bio cannot exceed 1000 characters");
        }
        self.bio = Some(value.clone());
        Ok(self)
    }
}

// Validation for avatar URL
impl diesel_builders::TrySetColumn<user_profiles::avatar_url> for NewUserProfile {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            anyhow::bail!("Avatar URL must start with http:// or https://");
        }
        if value.len() > 500 {
            anyhow::bail!("Avatar URL cannot exceed 500 characters");
        }
        self.avatar_url = Some(value.clone());
        Ok(self)
    }
}

impl TableAddition for user_profiles::table {
    type InsertableModel = NewUserProfile;
    type Model = UserProfile;
    type InsertableColumns = (
        user_profiles::id,
        user_profiles::bio,
        user_profiles::avatar_url,
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Create tables with CHECK constraints
    diesel::sql_query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 100),
            email TEXT NOT NULL CHECK(length(trim(email)) > 0)
        )",
    )
    .execute(&mut conn)?;

    diesel::sql_query(
        "CREATE TABLE user_profiles (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES users(id),
            bio TEXT NOT NULL CHECK(length(trim(bio)) > 0 AND length(bio) <= 1000),
            avatar_url TEXT NOT NULL CHECK((avatar_url LIKE 'http://%' OR avatar_url LIKE 'https://%') AND length(avatar_url) <= 500)
        )"
    )
    .execute(&mut conn)?;

    // Build and insert a valid profile (which also creates the parent user)
    let profile = user_profiles::table::builder()
        .try_set_column::<users::name>(&"Bob Smith".to_string())?
        .try_set_column::<users::email>(&"bob@example.com".to_string())?
        .try_set_column::<user_profiles::bio>(
            &"Software developer and open source enthusiast".to_string(),
        )?
        .try_set_column::<user_profiles::avatar_url>(&"https://example.com/avatar.jpg".to_string())?
        .insert(&mut conn)?;

    println!("Successfully inserted profile: {profile:?}");
    assert_eq!(profile.bio, "Software developer and open source enthusiast");
    assert_eq!(profile.avatar_url, "https://example.com/avatar.jpg");

    // Test validation - empty email
    let mut builder = user_profiles::table::builder();
    builder.try_set_column::<users::name>(&"Alice".to_string())?;
    let result = builder.try_set_column::<users::email>(&"   ".to_string());
    assert!(result.is_err());
    println!("Email validation works: {}", result.unwrap_err());

    // Test validation - invalid URL
    let mut builder = user_profiles::table::builder();
    builder.try_set_column::<users::name>(&"Charlie".to_string())?;
    builder.try_set_column::<users::email>(&"charlie@example.com".to_string())?;
    builder.try_set_column::<user_profiles::bio>(&"Developer".to_string())?;
    let result = builder
        .try_set_column::<user_profiles::avatar_url>(&"ftp://bad-url.com/avatar.jpg".to_string());
    assert!(result.is_err());
    println!("Avatar URL validation works: {}", result.unwrap_err());

    println!("All validations passed!");
    Ok(())
}
