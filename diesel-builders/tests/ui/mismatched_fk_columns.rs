use diesel_builders::prelude::*;

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
    }
}

#[derive(TableModel, Queryable, Selectable, Identifiable)]
#[diesel(table_name = posts)]
#[table_model(foreign_key((user_id, title), (users::id)))]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

fn main() {}
