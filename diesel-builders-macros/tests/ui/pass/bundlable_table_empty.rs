// Test that bundlable_table works with empty tuples

use diesel_builders_macros::bundlable_table;

struct TableA;

impl diesel::Table for TableA {
    type PrimaryKey = Column1;
    type AllColumns = Column1;

    fn all_columns() -> Self::AllColumns {
        Column1
    }
}

struct Column1;

trait BundlableTable {
    type MandatoryTriangularSameAsColumns;
    type DiscretionaryTriangularSameAsColumns;
}

#[bundlable_table]
impl BundlableTable for TableA {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = ();
}

fn main() {}
