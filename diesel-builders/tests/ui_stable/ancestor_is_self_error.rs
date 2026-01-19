use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(dogs))]
pub struct Dog {
    id: i32,
    breed: String,
}

fn main() {}
