// Test that descendant_of works with multiple ancestors

use diesel_builders_macros::descendant_of;

struct TableA;
struct TableB;
struct TableC;

impl diesel::Table for TableA {
    type PrimaryKey = ColumnA;
    type AllColumns = ColumnA;
    fn all_columns() -> Self::AllColumns {
        ColumnA
    }
}

impl diesel::Table for TableB {
    type PrimaryKey = ColumnB;
    type AllColumns = ColumnB;
    fn all_columns() -> Self::AllColumns {
        ColumnB
    }
}

impl diesel::Table for TableC {
    type PrimaryKey = ColumnC;
    type AllColumns = ColumnC;
    fn all_columns() -> Self::AllColumns {
        ColumnC
    }
}

struct ColumnA;
struct ColumnB;
struct ColumnC;

trait Descendant {
    type Ancestors;
    type Root;
}

#[descendant_of]
impl Descendant for TableC {
    type Ancestors = (TableA, TableB);
    type Root = TableA;
}

fn main() {}
