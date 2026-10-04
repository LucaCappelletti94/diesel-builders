use diesel::SqliteConnection;
use diesel_builders::prelude::*;

mod schema {
    use diesel_builders::{MayGetColumn, prelude::*};

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum ValidationError {
        #[error("Age must be at least eighteen")]
        AgeTooYoung,
        #[error("Nickname cannot be empty")]
        EmptyNickname,
        #[error("Value must be nonnegative")]
        NegativeValue,
        #[error("Label exceeds its permitted length")]
        LabelTooLong,
        #[error("Label is reserved by the parent")]
        ReservedLabel,
    }

    impl From<std::convert::Infallible> for ValidationError {
        fn from(error: std::convert::Infallible) -> Self {
            match error {}
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    #[error("Start must precede end")]
    pub struct RecordError;

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = users)]
    #[table_model(surrogate_key, error = ValidationError)]
    pub struct User {
        pub id: i32,
        #[infallible]
        pub name: String,
        #[table_model(default = 18)]
        pub age: i32,
        pub nickname: Option<String>,
    }

    impl ValidateColumn<users::age> for <users::table as TableExt>::NewValues {
        type Error = ValidationError;
        fn validate_column(value: &i32) -> Result<(), Self::Error> {
            if *value < 18 { Err(ValidationError::AgeTooYoung) } else { Ok(()) }
        }
    }

    impl ValidateColumn<users::nickname> for <users::table as TableExt>::NewValues {
        type Error = ValidationError;
        fn validate_column(value: &String) -> Result<(), Self::Error> {
            if value.is_empty() { Err(ValidationError::EmptyNickname) } else { Ok(()) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = posts)]
    #[table_model(surrogate_key, foreign_key(user_id, (users::id)))]
    pub struct Post {
        pub id: i32,
        pub user_id: i32,
        pub title: String,
    }

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = links)]
    #[table_model(surrogate_key, foreign_key(owner, (users::id)))]
    pub struct Link {
        pub id: i32,
        pub owner: i32,
    }

    diesel::allow_tables_to_appear_in_same_query!(users, links);

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = profiles)]
    #[table_model(ancestors(users))]
    pub struct Profile {
        pub id: i32,
        #[same_as(users::name)]
        pub display_name: String,
        pub visits: i32,
    }

    diesel::allow_tables_to_appear_in_same_query!(users, posts, profiles);

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = invalid_defaults)]
    #[table_model(surrogate_key, error = ValidationError)]
    pub struct InvalidDefault {
        pub id: i32,
        #[table_model(default = -1)]
        pub value: i32,
    }

    impl ValidateColumn<invalid_defaults::value> for <invalid_defaults::table as TableExt>::NewValues {
        type Error = ValidationError;
        fn validate_column(value: &i32) -> Result<(), Self::Error> {
            if *value < 0 { Err(ValidationError::NegativeValue) } else { Ok(()) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = intervals)]
    #[table_model(surrogate_key, record_error = RecordError)]
    pub struct Interval {
        pub id: i32,
        pub start: i32,
        pub end: i32,
    }

    impl ValidateRecord<intervals::table> for <intervals::table as TableExt>::NewValues {
        fn validate_record(&self) -> Result<(), RecordError> {
            let start = MayGetColumn::<intervals::start>::may_get_column(self);
            let end = MayGetColumn::<intervals::end>::may_get_column(self);
            match (start, end) {
                (Some(start), Some(end)) if start >= end => Err(RecordError),
                _ => Ok(()),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = parents)]
    #[table_model(surrogate_key)]
    pub struct Parent {
        pub id: i32,
        pub label: String,
    }

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = sides)]
    #[table_model(surrogate_key, foreign_key(parent_id, (parents::id)))]
    pub struct Side {
        pub id: i32,
        pub parent_id: i32,
        pub label: String,
    }

    unique_index!(sides::id, sides::parent_id);
    unique_index!(sides::id, sides::label);

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = children)]
    #[table_model(ancestors(parents))]
    pub struct Child {
        #[same_as(sides::parent_id, mandatory_id)]
        #[same_as(sides::parent_id, discretionary_id)]
        pub id: i32,
        #[same_as(parents::label)]
        #[same_as(sides::label, mandatory_id)]
        #[same_as(sides::label, discretionary_id)]
        pub child_label: String,
        #[mandatory(sides)]
        pub mandatory_id: i32,
        #[discretionary(sides)]
        pub discretionary_id: i32,
    }

    diesel::allow_tables_to_appear_in_same_query!(parents, sides, children);

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = checked_parents)]
    #[table_model(surrogate_key, error = ValidationError)]
    pub struct CheckedParent {
        pub id: i32,
        pub label: String,
    }

    impl ValidateColumn<checked_parents::label> for <checked_parents::table as TableExt>::NewValues {
        type Error = ValidationError;
        fn validate_column(value: &String) -> Result<(), Self::Error> {
            if value == "deny" {
                Err(ValidationError::ReservedLabel)
            } else if value.len() > 10 {
                Err(ValidationError::LabelTooLong)
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = checked_sides)]
    #[table_model(surrogate_key, error = ValidationError, foreign_key(parent_id, (checked_parents::id)))]
    pub struct CheckedSide {
        pub id: i32,
        #[infallible]
        pub parent_id: i32,
        pub label: String,
    }

    impl ValidateColumn<checked_sides::label> for <checked_sides::table as TableExt>::NewValues {
        type Error = ValidationError;
        fn validate_column(value: &String) -> Result<(), Self::Error> {
            if value.len() > 5 { Err(ValidationError::LabelTooLong) } else { Ok(()) }
        }
    }

    unique_index!(checked_sides::id, checked_sides::parent_id);
    unique_index!(checked_sides::id, checked_sides::label);

    #[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
    #[diesel(table_name = checked_children)]
    #[table_model(ancestors(checked_parents), error = ValidationError)]
    pub struct CheckedChild {
        #[same_as(checked_sides::parent_id, mandatory_id)]
        #[same_as(checked_sides::parent_id, discretionary_id)]
        #[infallible]
        pub id: i32,
        #[same_as(checked_parents::label)]
        #[same_as(checked_sides::label, mandatory_id)]
        #[same_as(checked_sides::label, discretionary_id)]
        pub child_label: String,
        #[mandatory(checked_sides)]
        #[infallible]
        pub mandatory_id: i32,
        #[discretionary(checked_sides)]
        #[infallible]
        pub discretionary_id: i32,
    }

    impl ValidateColumn<checked_children::child_label>
        for <checked_children::table as TableExt>::NewValues
    {
        type Error = ValidationError;
        fn validate_column(value: &String) -> Result<(), Self::Error> {
            if value.len() > 12 { Err(ValidationError::LabelTooLong) } else { Ok(()) }
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(checked_parents, checked_sides, checked_children);
}

fn connection() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:")?;
    diesel::sql_query("PRAGMA foreign_keys = ON").execute(&mut conn)?;
    for ddl in [
        "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL, age INTEGER NOT NULL CHECK(age >= 18), nickname TEXT)",
        "CREATE TABLE posts(id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id), title TEXT NOT NULL)",
        "CREATE TABLE links(id INTEGER PRIMARY KEY, owner INTEGER NOT NULL REFERENCES users(id))",
        "CREATE TABLE profiles(id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE, display_name TEXT NOT NULL, visits INTEGER NOT NULL CHECK(visits >= 0))",
        "CREATE TABLE invalid_defaults(id INTEGER PRIMARY KEY, value INTEGER NOT NULL CHECK(value >= 0))",
        "CREATE TABLE intervals(id INTEGER PRIMARY KEY, start INTEGER NOT NULL, end INTEGER NOT NULL CHECK(start < end))",
        "CREATE TABLE parents(id INTEGER PRIMARY KEY, label TEXT NOT NULL)",
        "CREATE TABLE sides(id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES parents(id), label TEXT NOT NULL, UNIQUE(id, parent_id), UNIQUE(id, label))",
        "CREATE TABLE children(id INTEGER PRIMARY KEY REFERENCES parents(id) ON DELETE CASCADE, child_label TEXT NOT NULL, mandatory_id INTEGER NOT NULL REFERENCES sides(id), discretionary_id INTEGER NOT NULL REFERENCES sides(id))",
        "CREATE TABLE checked_parents(id INTEGER PRIMARY KEY, label TEXT NOT NULL)",
        "CREATE TABLE checked_sides(id INTEGER PRIMARY KEY, parent_id INTEGER NOT NULL REFERENCES checked_parents(id), label TEXT NOT NULL, UNIQUE(id, parent_id), UNIQUE(id, label))",
        "CREATE TABLE checked_children(id INTEGER PRIMARY KEY REFERENCES checked_parents(id), child_label TEXT NOT NULL, mandatory_id INTEGER NOT NULL REFERENCES checked_sides(id), discretionary_id INTEGER NOT NULL REFERENCES checked_sides(id))",
    ] {
        diesel::sql_query(ddl).execute(&mut conn)?;
    }
    Ok(conn)
}

fn connection_with_data() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = connection()?;
    diesel::sql_query("INSERT INTO users(id, name, age, nickname) VALUES (1, 'Ada', 20, 'Ace'), (2, 'Grace', 30, NULL)").execute(&mut conn)?;
    diesel::sql_query("INSERT INTO posts(id, user_id, title) VALUES (1, 1, 'First'), (2, 1, 'Second'), (3, 2, 'Third')").execute(&mut conn)?;
    diesel::sql_query("INSERT INTO profiles(id, display_name, visits) VALUES (1, 'Ada', 3)")
        .execute(&mut conn)?;
    Ok(conn)
}

fn connection_with_profiles() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    let mut conn = connection_with_data()?;
    diesel::sql_query("INSERT INTO users(id, name, age) VALUES (3, 'Bob', 25), (4, 'Edsger', 40)")
        .execute(&mut conn)?;
    diesel::sql_query("INSERT INTO profiles(id, display_name, visits) VALUES (3, 'Bob', 3), (2, 'Grace', 3), (4, 'Edsger', 5)").execute(&mut conn)?;
    Ok(conn)
}

fn user_values(
    name: &str,
    age: i32,
    nickname: Option<&str>,
) -> <schema::users::table as TableExt>::NewValues {
    use diesel_builders::SetColumn;
    let mut values = schema::users::table::empty_new_values();
    SetColumn::<schema::users::name>::set_column(&mut values, name);
    SetColumn::<schema::users::age>::set_column(&mut values, age);
    SetColumn::<schema::users::nickname>::set_column(&mut values, nickname.map(str::to_owned));
    values
}
