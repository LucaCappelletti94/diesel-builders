// Test that descendant_of fails when Ancestors type is missing

use diesel_builders_macros::descendant_of;

struct TableA;

trait Descendant {
    type Ancestors;
    type Root;
}

#[descendant_of]
impl Descendant for TableA {
    type Root = Self;
}

fn main() {}
