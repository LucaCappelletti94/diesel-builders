// Test that SetColumn derive fails without diesel attribute

use diesel_builders_macros::SetColumn;

#[derive(SetColumn)]
struct NewUser {
    id: Option<i32>,
    name: Option<String>,
}

fn main() {}
