use diesel_builders::prelude::*;

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = mandatory_table)]
pub struct Mandatory {
    id: i32,
    parent_id: i32,
    mandatory_field: String,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = child_table)]
pub struct Child {
    // Missing #[same_as(mandatory_table::parent_id)]
    id: i32,
    #[mandatory(mandatory_table)]
    mandatory_id: i32,
}

fn main() {}
