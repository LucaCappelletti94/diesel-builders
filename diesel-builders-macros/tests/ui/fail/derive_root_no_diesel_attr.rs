// Test that Root derive fails without diesel attribute

use diesel_builders_macros::Root;

#[derive(Root)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
