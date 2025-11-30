// Test that Root derive works

use diesel_builders_macros::Root;

mod table_a {
    pub struct table;
    pub struct id;
    pub struct name;
}

#[derive(Root)]
#[diesel(table_name = table_a)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
