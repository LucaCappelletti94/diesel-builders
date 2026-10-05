use diesel_builders::prelude::*;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(foreign_key(other_id, ()))]
pub struct Child {
    id: i32,
    other_id: i32,
}

fn main() {}
