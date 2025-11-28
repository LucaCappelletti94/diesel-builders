//! Submodule to test whether the diesel-builder can work in the base case
//! of a single table with no ancestors and no vertical/horizontal same-as
//! relationships.

use diesel::{associations::HasTable, prelude::*, sqlite::SqliteConnection};
use diesel_additions::{GetColumn, MayGetColumn, TableAddition, TrySetColumn, TypedColumn};
use diesel_builders::{BuildableTable, BundlableTable, NestedInsert};
use diesel_relations::{AncestorOfIndex, Descendant};
use typed_tuple::prelude::TupleIndex0;

table! {
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

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq)]
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

impl AncestorOfIndex<users::table> for users::table {
    type Idx = TupleIndex0;
}

impl Descendant for users::table {
    type Ancestors = ();
    type Root = Self;
}

#[derive(Debug, Default, Clone, Insertable)]
#[diesel(table_name = users)]
/// A new user model for insertions.
pub struct NewUser {
    /// The name of the user.
    pub name: Option<String>,
    /// The email of the user.
    pub email: Option<String>,
}

impl HasTable for NewUser {
    type Table = users::table;

    fn table() -> Self::Table {
        users::table
    }
}

impl TableAddition for users::table {
    type InsertableModel = NewUser;
    type Model = User;
    type InsertableColumns = (users::name, users::email);
}

impl TypedColumn for users::id {
    type Type = i32;
}

impl TypedColumn for users::name {
    type Type = String;
}

impl TypedColumn for users::email {
    type Type = String;
}

impl BundlableTable for users::table {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = ();
}

impl GetColumn<users::id> for User {
    fn get_column(&self) -> &i32 {
        &self.id
    }
}

impl GetColumn<users::name> for User {
    fn get_column(&self) -> &String {
        &self.name
    }
}

impl GetColumn<users::email> for User {
    fn get_column(&self) -> &String {
        &self.email
    }
}

impl MayGetColumn<users::name> for NewUser {
    fn may_get_column(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

impl MayGetColumn<users::email> for NewUser {
    fn may_get_column(&self) -> Option<&String> {
        self.email.as_ref()
    }
}

impl TrySetColumn<users::name> for NewUser {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<()> {
        self.name = Some(value.clone());
        Ok(())
    }
}

impl TrySetColumn<users::email> for NewUser {
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<()> {
        self.email = Some(value.clone());
        Ok(())
    }
}

#[test]
fn test_simple_table() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;

    diesel::sql_query(
        "CREATE TABLE users (
			id INTEGER PRIMARY KEY NOT NULL,
			name TEXT NOT NULL,
			email TEXT NOT NULL
		)",
    )
    .execute(&mut conn)?;

    let mut builder = <users::table as BuildableTable>::builder();
    TrySetColumn::<users::name>::try_set_column(&mut builder, &"Alice".to_string())?;
    TrySetColumn::<users::email>::try_set_column(&mut builder, &"alice@example.com".to_string())?;
    let user = builder.insert(&mut conn)?;

    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");

    // We attempt to query the inserted user to ensure everything worked correctly.
    let queried_user: User = users::table.filter(users::id.eq(user.id)).first(&mut conn)?;
    assert_eq!(user, queried_user);

    Ok(())
}
