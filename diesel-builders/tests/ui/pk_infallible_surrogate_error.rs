use diesel_builders::prelude::*;

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

#[derive(TableModel)]
#[diesel(table_name = users)]
#[table_model(surrogate_key)]
pub struct User {
    #[table_model(infallible)]
    pub id: i32,
    pub name: String,
}

fn main() {}
