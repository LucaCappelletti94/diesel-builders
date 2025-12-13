use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = root_table)]
#[table_model(surrogate_key)]
pub struct Root {
    id: i32,
    data: String,
}

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
#[diesel(primary_key(id))]
#[table_model(ancestors(root_table))]
pub struct Child {
    id: i64,  // Different type from root's i32!
    child_data: String,
}

fn main() {}
