use diesel_builders::prelude::*;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(ancestors())]
pub struct Child {
    id: i32,
    name: String,
}

fn main() {}
