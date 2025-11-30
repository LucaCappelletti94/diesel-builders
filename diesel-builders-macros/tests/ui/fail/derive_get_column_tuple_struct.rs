// Test that GetColumn derive fails on tuple struct

use diesel_builders_macros::GetColumn;

mod table_a {
    pub struct table;
}

#[derive(GetColumn)]
#[diesel(table_name = table_a)]
struct User(i32, String);

fn main() {}
