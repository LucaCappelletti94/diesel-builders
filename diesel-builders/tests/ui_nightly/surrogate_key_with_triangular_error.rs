use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = parent_table)]
#[table_model(surrogate_key)]
pub struct Parent {
    id: i32,
    #[mandatory]
    parent_field: i32,
}

fn main() {}
