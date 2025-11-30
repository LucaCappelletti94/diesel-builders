// Test that descendant_of works with single ancestor

use diesel_builders_macros::descendant_of;

struct TableA;
struct TableB;

impl diesel::Table for TableA {
    type PrimaryKey = ColumnA1;
    type AllColumns = ColumnA1;

    fn all_columns() -> Self::AllColumns {
        ColumnA1
    }
}

impl diesel::Table for TableB {
    type PrimaryKey = ColumnB1;
    type AllColumns = ColumnB1;

    fn all_columns() -> Self::AllColumns {
        ColumnB1
    }
}

struct ColumnA1;
struct ColumnB1;

trait Descendant {
    type Ancestors;
    type Root;
}

#[descendant_of]
impl Descendant for TableB {
    type Ancestors = (TableA,);
    type Root = TableA;
}

fn main() {}
