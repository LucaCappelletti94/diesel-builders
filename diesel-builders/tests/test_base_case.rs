//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

mod common;

use diesel::prelude::*;
use diesel_additions::{
    GetColumnExt, MayGetColumnExt, SetColumnExt, TableAddition, TrySetColumnExt,
};
use diesel_builders::{BuildableTable, BundlableTable, NestedInsert};
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel};

diesel::table! {
    /// Define a simple users table for testing.
    users (id) {
        /// The ID of the user.
        id -> Integer,
        /// The name of the user.
        name -> Text,
        /// The email of the user.
        email -> Text,
    }
}

#[derive(
    Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, GetColumn, Root, TableModel,
)]
#[diesel(table_name = users)]
/// A simple user model.
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

#[test]
fn test_simple_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    diesel::sql_query(
        "CREATE TABLE users (
			id INTEGER PRIMARY KEY NOT NULL,
			name TEXT NOT NULL,
			email TEXT NOT NULL
		)",
    )
    .execute(&mut conn)?;

    let mut builder = users::table::builder();

    assert_eq!(builder.may_get_column::<users::name>(), None);
    assert_eq!(builder.may_get_column::<users::email>(), None);

    builder.try_set_column::<users::name>(&"Alice".to_string())?;

    assert_eq!(builder.may_get_column::<users::name>(), Some(&"Alice".to_string()));
    assert_eq!(builder.may_get_column::<users::email>(), None);

    builder.try_set_column::<users::email>(&"alice@example.com".to_string())?;

    assert_eq!(builder.may_get_column::<users::name>(), Some(&"Alice".to_string()));
    assert_eq!(builder.may_get_column::<users::email>(), Some(&"alice@example.com".to_string()));

    let user = builder.insert(&mut conn)?;

    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");

    assert_eq!(user.get_column::<users::name>(), &"Alice".to_string());
    assert_eq!(user.get_column::<users::email>(), &"alice@example.com".to_string());

    // We attempt to query the inserted user to ensure everything worked correctly.
    let queried_user: User = users::table.filter(users::id.eq(user.id)).first(&mut conn)?;
    assert_eq!(user, queried_user);

    // We test the chained variant.
    let another_user = users::table::builder()
        .set_column::<users::name>(&"Bob".to_string())
        .set_column::<users::email>(&"bob@example.com".to_string())
        .insert(&mut conn)?;

    assert_eq!(another_user.get_column::<users::name>(), &"Bob".to_string());
    assert_eq!(another_user.get_column::<users::email>(), &"bob@example.com".to_string());

    assert_ne!(user.get_column::<users::id>(), another_user.get_column::<users::id>());

    Ok(())
}
