use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = animals)]
pub struct Animal {
    id: i32,
    name: String,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(animals, animals))]
pub struct Dog {
    id: i32,
    breed: String,
}

fn main() {}
