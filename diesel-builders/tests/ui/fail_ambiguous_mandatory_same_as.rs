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

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
pub struct Child {
    #[same_as(mandatory_table::parent_id)]
    id: i32,
    #[mandatory(mandatory_table)]
    mand_id_1: i32,
    #[mandatory(mandatory_table)]
    mand_id_2: i32,
    #[same_as(mandatory_table::mandatory_field)]
    col1: String,
}

fn main() {
    // This should fail because HorizontalKey is not implemented for mand_id_1 or mand_id_2
    // due to ambiguity.
    fn check_trait<T: diesel_builders::HorizontalKey>() {}
    check_trait::<child_table::columns::mand_id_1>();
}
