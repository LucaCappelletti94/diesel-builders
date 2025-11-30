// Test that Root derive fails on tuple struct

use diesel_builders_macros::Root;

mod table_a {
    pub struct table;
}

#[derive(Root)]
#[diesel(table_name = table_a)]
struct User(i32, String);

fn main() {}
