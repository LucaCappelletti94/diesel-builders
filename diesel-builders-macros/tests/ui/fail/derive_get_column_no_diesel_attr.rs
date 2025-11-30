// Test that GetColumn derive fails without diesel attribute

use diesel_builders_macros::GetColumn;

#[derive(GetColumn)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
