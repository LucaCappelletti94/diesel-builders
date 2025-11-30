// Test that MayGetColumn derive fails without diesel attribute

use diesel_builders_macros::MayGetColumn;

#[derive(MayGetColumn)]
struct NewUser {
    id: Option<i32>,
    name: Option<String>,
}

fn main() {}
