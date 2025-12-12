use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = test_table)]
pub struct TestModel {
    id: i32,
    #[mandatory]
    #[mandatory]
    field1: i32,
}

fn main() {}
