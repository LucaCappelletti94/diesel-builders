use diesel_builders::prelude::*;

diesel::table! {
    users (id, name) {
        id -> Integer,
        name -> Text,
    }
}

#[derive(TableModel)]
#[diesel(table_name = users)]
#[diesel(primary_key(id, name))]
#[table_model(surrogate_key)]
pub struct User {
    pub id: i32,
    pub name: String,
}

fn main() {}
