// Test that HasTable derive works

use diesel_builders_macros::HasTable;

mod table_a {
    pub struct table;
}

#[derive(HasTable)]
#[diesel(table_name = table_a)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
