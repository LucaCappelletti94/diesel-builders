// Test that HasTable derive fails without diesel attribute

use diesel_builders_macros::HasTable;

#[derive(HasTable)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
