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
#[diesel(table_name = mandatory_table)]
pub struct Mandatory {
    id: i32,
    parent_id: i32,
    mandatory_field: String,
}

unique_index!(mandatory_table::id, mandatory_table::parent_id);
unique_index!(mandatory_table::id, mandatory_table::mandatory_field);

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
pub struct Child {
    #[same_as(mandatory_table::parent_id)]
    id: i32,
    #[mandatory(mandatory_table)]
    mandatory_id: i32,
    #[same_as(mandatory_table::mandatory_field)]
    col1: String,
    #[same_as(mandatory_table::mandatory_field)]
    col2: String,
}

fn main() {}
