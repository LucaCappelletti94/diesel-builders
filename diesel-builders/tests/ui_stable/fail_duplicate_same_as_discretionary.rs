use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = parent_table)]
pub struct Parent {
    id: i32,
    parent_field: String,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(surrogate_key)]
#[diesel(table_name = discretionary_table)]
pub struct Discretionary {
    id: i32,
    parent_id: i32,
    discretionary_field: String,
}

unique_index!(discretionary_table::id, discretionary_table::parent_id);
unique_index!(discretionary_table::id, discretionary_table::discretionary_field);

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
pub struct Child {
    #[same_as(discretionary_table::parent_id)]
    id: i32,
    #[discretionary(discretionary_table)]
    discretionary_id: i32,
    #[same_as(discretionary_table::discretionary_field)]
    col1: String,
    #[same_as(discretionary_table::discretionary_field)]
    col2: String,
}

fn main() {}
