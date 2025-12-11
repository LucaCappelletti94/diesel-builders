use diesel_builders::prelude::*;

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct User {
    #[table_model(default = 1)]
    pub id: i32,
    pub name: String,
}

fn main() {}
