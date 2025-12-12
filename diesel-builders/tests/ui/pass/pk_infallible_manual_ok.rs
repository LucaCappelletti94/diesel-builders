use diesel_builders::prelude::*;

#[derive(TableModel, Identifiable, Queryable, Selectable, Debug, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    #[table_model(infallible)]
    pub id: i32,
    pub name: String,
}

fn main() {}
