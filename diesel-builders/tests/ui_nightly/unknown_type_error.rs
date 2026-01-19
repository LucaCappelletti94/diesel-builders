use diesel_builders::prelude::*;

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub data: std::collections::HashMap<String, String>,
}

fn main() {}
