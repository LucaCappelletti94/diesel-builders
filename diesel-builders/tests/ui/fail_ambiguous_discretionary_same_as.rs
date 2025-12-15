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

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[table_model(ancestors = parent_table)]
#[diesel(table_name = child_table)]
pub struct Child {
    #[same_as(discretionary_table::parent_id)]
    id: i32,
    #[discretionary(discretionary_table)]
    disc_id_1: i32,
    #[discretionary(discretionary_table)]
    disc_id_2: i32,
    #[same_as(discretionary_table::discretionary_field)]
    col1: String,
}

fn main() {
    // This should fail because HorizontalKey is not implemented for disc_id_1 or disc_id_2
    // due to ambiguity, so we can't use the builder to set col1 via the relationship.
    // We simulate this by checking for the trait implementation.
    fn check_trait<T: diesel_builders::HorizontalKey>() {}
    check_trait::<child_table::columns::disc_id_1>();
}
