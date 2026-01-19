use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(surrogate_key)]
pub struct Child {
    id: i32,
    #[discretionary]
    optional_field: i32,
}

fn main() {}
