use diesel_builders::prelude::*;

#[derive(TableModel)]
#[diesel(table_name = users)]
#[table_model(surrogate_key)]
pub struct User {
    #[table_model(default = 1)]
    pub id: i32,
    pub name: String,
}

fn main() {}
