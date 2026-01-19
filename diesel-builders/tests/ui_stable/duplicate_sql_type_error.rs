use diesel_builders::prelude::*;

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub duplicate: i32,
}

fn main() {}
