//! Test for custom types.

mod shared;
use diesel::prelude::*;
use diesel_builders::prelude::*;

/// Custom SQL type
#[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
#[diesel(sqlite_type(name = "Integer"))]
pub struct MyCustomSqlType;

/// Custom Rust type
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    diesel::expression::AsExpression,
    diesel::deserialize::FromSqlRow,
)]
#[diesel(sql_type = MyCustomSqlType)]
pub struct MyCustomType {
    /// Foo
    pub foo: i32,
}

// Implement ToSql for SQLite
impl diesel::serialize::ToSql<MyCustomSqlType, diesel::sqlite::Sqlite> for MyCustomType {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        out.set_value(self.foo);
        Ok(diesel::serialize::IsNull::No)
    }
}

// Implement FromSql for SQLite
impl diesel::deserialize::FromSql<MyCustomSqlType, diesel::sqlite::Sqlite> for MyCustomType {
    fn from_sql(bytes: diesel::sqlite::SqliteValue) -> diesel::deserialize::Result<Self> {
        let val = <i32 as diesel::deserialize::FromSql<
            diesel::sql_types::Integer,
            diesel::sqlite::Sqlite,
        >>::from_sql(bytes)?;
        Ok(MyCustomType { foo: val })
    }
}

/// User model
#[derive(
    Debug, Clone, PartialEq, diesel::Identifiable, diesel::Queryable, diesel::Selectable, TableModel,
)]
#[diesel(table_name = users)]
pub struct User {
    /// ID
    id: i32,
    /// Custom field
    // Use crate:: because this is an integration test and the type is at the root
    #[diesel(sql_type = crate::MyCustomSqlType)]
    custom_field: MyCustomType,
    /// Optional custom field
    // Use crate:: because this is an integration test and the type is at the root
    #[diesel(sql_type = crate::MyCustomSqlType)]
    another_custom_field: Option<MyCustomType>,
    /// Binary data
    binary_data: Vec<u8>,
}

#[test]
fn test_custom_type_compiles() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;

    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, custom_field INTEGER NOT NULL, another_custom_field INTEGER, binary_data BLOB NOT NULL)")
        .execute(&mut conn)?;

    let user = users::table::builder()
        .id(1)
        .custom_field(MyCustomType { foo: 42 })
        .binary_data(vec![1, 2, 3])
        .insert(&mut conn)?;

    let retrieved_user = users::table.find(1).first::<User>(&mut conn)?;

    assert_eq!(user, retrieved_user);
    assert_eq!(user.custom_field, MyCustomType { foo: 42 });
    assert_eq!(user.binary_data, vec![1, 2, 3]);

    Ok(())
}
