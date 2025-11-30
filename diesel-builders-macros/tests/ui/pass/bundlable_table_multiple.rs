// Test that bundlable_table works with multiple columns

use diesel_builders_macros::bundlable_table;

struct TableA;
struct Column1;
struct Column2;
struct Column3;
struct Column4;

impl diesel::Table for TableA {
    type PrimaryKey = Column1;
    type AllColumns = (Column1, Column2, Column3, Column4);

    fn all_columns() -> Self::AllColumns {
        (Column1, Column2, Column3, Column4)
    }
}

trait BundlableTable {
    type MandatoryTriangularSameAsColumns;
    type DiscretionaryTriangularSameAsColumns;
}

#[bundlable_table]
impl BundlableTable for TableA {
    type MandatoryTriangularSameAsColumns = (Column2, Column3);
    type DiscretionaryTriangularSameAsColumns = (Column4,);
}

fn main() {}
