use diesel::prelude::*;
use diesel_builders::prelude::*;

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, PartialOrd, TableModel)]
#[diesel(table_name = dogs)]
#[table_model(ancestors(dogs::table))]
pub struct Dog {
    id: i32,
    breed: String,
}

fn main() {}
