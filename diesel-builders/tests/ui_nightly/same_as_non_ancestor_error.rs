use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = parent_table)]
pub struct Parent {
    id: i32,
    name: String,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = unrelated_table)]
pub struct Unrelated {
    id: i32,
    field: String,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
pub struct Child {
    id: i32,
    #[same_as(unrelated_table::field)]
    child_field: String,
}

fn main() {}
