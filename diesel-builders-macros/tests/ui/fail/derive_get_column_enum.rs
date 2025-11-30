// Test that GetColumn derive fails on enum

use diesel_builders_macros::GetColumn;

mod table_a {
    pub struct table;
}

#[derive(GetColumn)]
#[diesel(table_name = table_a)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
