// Test that MayGetColumn derive works

use diesel_builders_macros::MayGetColumn;

mod table_a {
    pub struct table;
    pub struct id;
    pub struct name;
}

#[derive(MayGetColumn)]
#[diesel(table_name = table_a)]
struct NewUser {
    id: Option<i32>,
    name: Option<String>,
}

fn main() {}
