// Test that bundlable_table works with single column

use diesel_builders_macros::bundlable_table;

struct TableA;
struct Column1;
struct Column2;

impl diesel::Table for TableA {
    type PrimaryKey = Column1;
    type AllColumns = (Column1, Column2);

    fn all_columns() -> Self::AllColumns {
        (Column1, Column2)
    }
}

trait BundlableTable {
    type MandatoryTriangularSameAsColumns;
    type DiscretionaryTriangularSameAsColumns;
}

#[bundlable_table]
impl BundlableTable for TableA {
    type MandatoryTriangularSameAsColumns = (Column2,);
    type DiscretionaryTriangularSameAsColumns = ();
}

fn main() {}
