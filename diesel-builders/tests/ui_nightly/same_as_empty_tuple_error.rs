use diesel_builders::prelude::*;
use diesel_builders_derive::TableModel;

mod other_table {
    use diesel::prelude::*;
    diesel::table! {
        other_table(id) {
            id -> Integer,
        }
    }
}

#[derive(TableModel)]
#[diesel(table_name = my_table)]
pub struct MyTable {
    #[same_as(other_table::id, ())]
    id: i32,
}

fn main() {}
