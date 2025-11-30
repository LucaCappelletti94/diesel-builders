// Test that NoHorizontalSameAsGroup derive fails on enum

use diesel_builders_macros::NoHorizontalSameAsGroup;

mod table_a {
    pub struct table;
}

#[derive(NoHorizontalSameAsGroup)]
#[diesel(table_name = table_a)]
enum Status {
    Active,
    Inactive,
}

fn main() {}
