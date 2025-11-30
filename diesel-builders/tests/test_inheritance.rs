//! Test case for foreign key based inheritance where UserProfiles extends
//! Users. The primary key of UserProfiles is a foreign key to the primary key
//! of Users.

use diesel::{prelude::*, sqlite::SqliteConnection};
use diesel_additions::{GetColumnExt, SetColumnExt, TableAddition};
use diesel_builders::{BuildableTable, BundlableTable, NestedInsert};
use diesel_builders_macros::{
    GetColumn, HasTable, MayGetColumn, NoHorizontalSameAsGroup, Root, SetColumn, TableModel,
};
use diesel_relations::Descendant;

diesel::table! {
    /// Define a users table as the base/ancestor table.
    users (id) {
        /// The ID of the user.
        id -> Integer,
        /// The name of the user.
        name -> Text,
        /// The email of the user.
        email -> Text,
    }
}

diesel::table! {
    /// Define a user_profiles table that extends users via foreign key.
    user_profiles (id) {
        /// The ID of the user profile, which is also a foreign key to users.id.
        id -> Integer,
        /// The bio of the user.
        bio -> Text,
        /// The avatar URL of the user.
        avatar_url -> Text,
    }
}

// Define the join relationship between the tables
diesel::joinable!(user_profiles -> users (id));

// Allow tables to appear together in queries
diesel::allow_tables_to_appear_in_same_query!(users, user_profiles);

// Users table models

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
#[diesel(table_name = users)]
/// A user model.
pub struct User {
    /// The ID of the user.
    pub id: i32,
    /// The name of the user.
    pub name: String,
    /// The email of the user.
    pub email: String,
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = users)]
/// A new user model for insertions.
pub struct NewUser {
    /// The name of the user.
    pub name: Option<String>,
    /// The email of the user.
    pub email: Option<String>,
}

impl TableAddition for users::table {
    type InsertableModel = NewUser;
    type Model = User;
    type InsertableColumns = (users::name, users::email);
}

// UserProfiles table models

#[derive(
    Debug,
    Queryable,
    Clone,
    Selectable,
    Identifiable,
    PartialEq,
    GetColumn,
    TableModel,
    NoHorizontalSameAsGroup,
)]
#[diesel(table_name = user_profiles)]
/// A user profile model.
pub struct UserProfile {
    /// The ID of the user profile (foreign key to users.id).
    pub id: i32,
    /// The bio of the user.
    pub bio: String,
    /// The avatar URL of the user.
    pub avatar_url: String,
}

#[diesel_builders_macros::descendant_of]
impl Descendant for user_profiles::table {
    type Ancestors = (users::table,);
    type Root = users::table;
}

#[derive(Debug, Default, Clone, Insertable, MayGetColumn, SetColumn, HasTable)]
#[diesel(table_name = user_profiles)]
/// A new user profile model for insertions.
pub struct NewUserProfile {
    /// The ID of the user profile (should be set to match the user's ID).
    pub id: Option<i32>,
    /// The bio of the user.
    pub bio: Option<String>,
    /// The avatar URL of the user.
    pub avatar_url: Option<String>,
}

impl TableAddition for user_profiles::table {
    type InsertableModel = NewUserProfile;
    type Model = UserProfile;
    type InsertableColumns = (user_profiles::id, user_profiles::bio, user_profiles::avatar_url);
}

impl BundlableTable for user_profiles::table {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = ();
}

#[test]
fn test_inheritance() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    // Create the users table
    diesel::sql_query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // Create the user_profiles table with foreign key to users
    diesel::sql_query(
        "CREATE TABLE user_profiles (
            id INTEGER PRIMARY KEY NOT NULL REFERENCES users(id),
            bio TEXT NOT NULL,
            avatar_url TEXT NOT NULL
        )",
    )
    .execute(&mut conn)?;

    // We create a user without a profile
    let user = users::table::builder()
        .set_column::<users::name>(&"Bob".to_string())
        .set_column::<users::email>(&"bob@example.com".to_string())
        .insert(&mut conn)?;

    let loaded_user: User = users::table.filter(users::id.eq(user.id)).first(&mut conn)?;
    assert_eq!(loaded_user, user);

    // Now create a user profile for this user
    let profile = user_profiles::table::builder()
        .set_column::<users::name>(&"Alice".to_string())
        .set_column::<users::email>(&"alice@example.com".to_string())
        .set_column::<user_profiles::bio>(&"I love Rust!".to_string())
        .set_column::<user_profiles::avatar_url>(&"https://example.com/alice.jpg".to_string())
        .insert(&mut conn)?;

    assert_eq!(profile.bio, "I love Rust!");
    assert_eq!(profile.avatar_url, "https://example.com/alice.jpg");

    // Verify the profile can be queried
    let queried_profile: UserProfile =
        user_profiles::table.filter(user_profiles::id.eq(profile.id)).first(&mut conn)?;
    assert_eq!(profile, queried_profile);

    // Verify we can join the tables
    let (loaded_user, loaded_profile): (User, UserProfile) = users::table
        .inner_join(user_profiles::table)
        .filter(users::id.eq(profile.id))
        .first(&mut conn)?;

    assert_eq!(loaded_user.get_column::<users::id>(), &profile.id);
    assert_eq!(loaded_user.get_column::<users::name>(), "Alice");
    assert_eq!(loaded_profile, profile);

    Ok(())
}
