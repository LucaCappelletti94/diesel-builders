// Test that Root derive fails on enum

use diesel_builders_macros::Root;

mod table_a {
    pub struct table;
}

#[derive(Root)]
#[diesel(table_name = table_a)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
