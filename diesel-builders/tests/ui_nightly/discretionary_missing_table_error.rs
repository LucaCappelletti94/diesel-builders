use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = test_table)]
#[table_model(ancestors = parent_table)]
pub struct TestModel {
    id: i32,
    #[discretionary]
    field1: i32,
}

fn main() {}
