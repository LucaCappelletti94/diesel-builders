// Test that bundlable_table works with empty tuples

use diesel_builders_macros::bundlable_table;

diesel::table! {
    table_a (id) {
        id -> Integer,
    }
}

trait BundlableTable {
    type MandatoryTriangularSameAsColumns;
    type DiscretionaryTriangularSameAsColumns;
}

#[bundlable_table]
impl BundlableTable for table_a::table {
    type MandatoryTriangularSameAsColumns = ();
    type DiscretionaryTriangularSameAsColumns = ();
}

fn main() {}
