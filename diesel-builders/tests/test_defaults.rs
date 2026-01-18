//! Test defaults

mod shared;
use diesel::prelude::*;
use diesel_builders::prelude::*;

/// User model
#[derive(Debug, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = users)]
#[table_model(surrogate_key)]
pub struct User {
    /// Id
    pub id: i32,
    /// Name
    #[table_model(default = "Guest")]
    pub name: String,
    /// Role
    #[table_model(default = "User")]
    pub role: String,
    /// Active
    #[table_model(default = true)]
    pub active: bool,
    /// Bio
    // bio is Nullable<Text>, so Option<String>.
    // Default should be Some(None) (NULL) if not specified.
    pub bio: Option<String>,
    /// Email
    pub email: String,
}

#[test]
fn test_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query(
        "CREATE TABLE users (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        role TEXT NOT NULL,
        active BOOLEAN NOT NULL,
        bio TEXT,
        email TEXT NOT NULL
    )",
    )
    .execute(&mut conn)?;

    // Create builder
    let builder = users::table::builder();

    // Check defaults in builder
    assert_eq!(builder.may_get_column::<users::name>(), Some("Guest".to_string()));
    assert_eq!(builder.may_get_column::<users::role>(), Some("User".to_string()));
    assert_eq!(builder.may_get_column::<users::active>(), Some(true));

    // For nullable column `bio`, default is `Some(None)`.
    // `may_get_column` returns `Option<Option<String>>`.
    assert_eq!(builder.may_get_column::<users::bio>(), Some(None));

    // For non-nullable column `email` without default, default is `None`.
    assert_eq!(builder.may_get_column::<users::email>(), None);

    // Insert should fail because email is missing
    let res = builder.clone().insert(&mut conn);
    let err = res.unwrap_err();

    assert_eq!(err.to_string(), "Missing mandatory field: `users.email`");

    assert!(
        matches!(
            err,
            diesel_builders::BuilderError::Incomplete(
                diesel_builders::builder_error::IncompleteBuilderError::MissingMandatoryField {
                    table_name: "users",
                    field_name: "email",
                }
            )
        ),
        "Expected Incomplete error due to missing email field, got: {err:?}",
    );

    // Set email and insert
    let user = builder.try_email("test@example.com".to_string())?.insert(&mut conn)?;

    assert_eq!(user.name, "Guest");
    assert_eq!(user.role, "User");
    assert!(user.active);
    assert_eq!(user.bio, None);
    assert_eq!(user.email, "test@example.com");

    // Override defaults
    let user2 = users::table::builder()
        .try_name("Admin".to_string())?
        .try_bio(Some("Bio".to_string()))?
        .try_email("admin@example.com".to_string())?
        .insert(&mut conn)?;

    assert_eq!(user2.name, "Admin");
    assert_eq!(user2.role, "User"); // Default preserved
    assert_eq!(user2.bio, Some("Bio".to_string()));
    assert_eq!(user2.email, "admin@example.com");

    Ok(())
}
