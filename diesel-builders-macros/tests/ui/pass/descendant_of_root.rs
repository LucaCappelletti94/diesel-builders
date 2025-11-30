// Test that descendant_of works for root table (no ancestors)

use diesel_builders_macros::descendant_of;

struct TableA;

impl diesel::Table for TableA {
    type PrimaryKey = Column1;
    type AllColumns = Column1;

    fn all_columns() -> Self::AllColumns {
        Column1
    }
}

struct Column1;

trait Descendant {
    type Ancestors;
    type Root;
}

#[descendant_of]
impl Descendant for TableA {
    type Ancestors = ();
    type Root = Self;
}

fn main() {}
