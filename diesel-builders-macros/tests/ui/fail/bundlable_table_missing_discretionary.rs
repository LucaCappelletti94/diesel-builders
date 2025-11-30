// Test that bundlable_table fails when DiscretionaryTriangularSameAsColumns is missing

use diesel_builders_macros::bundlable_table;

struct TableA;

trait BundlableTable {
    type MandatoryTriangularSameAsColumns;
    type DiscretionaryTriangularSameAsColumns;
}

#[bundlable_table]
impl BundlableTable for TableA {
    type MandatoryTriangularSameAsColumns = ();
}

fn main() {}
