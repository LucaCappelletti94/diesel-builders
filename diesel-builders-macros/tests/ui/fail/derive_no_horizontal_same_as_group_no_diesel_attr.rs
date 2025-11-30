// Test that NoHorizontalSameAsGroup derive fails without diesel attribute

use diesel_builders_macros::NoHorizontalSameAsGroup;

#[derive(NoHorizontalSameAsGroup)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
