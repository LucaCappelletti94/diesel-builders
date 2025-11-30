// Test that TableModel derive fails without diesel attribute

use diesel_builders_macros::TableModel;

#[derive(TableModel)]
struct User {
    id: i32,
    name: String,
}

fn main() {}
