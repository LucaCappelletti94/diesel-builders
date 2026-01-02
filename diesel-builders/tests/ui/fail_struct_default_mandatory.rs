use diesel_builders::prelude::*;

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = satellite)]
#[diesel(primary_key(id))]
#[table_model(surrogate_key)]
pub struct Satellite {
    id: i32,
    parent_id: i32,
    info: String,
}

unique_index!(satellite::id, satellite::parent_id);

#[derive(Queryable, Selectable, Identifiable, TableModel)]
#[diesel(table_name = main_table)]
#[diesel(primary_key(id))]
#[table_model(default(satellite::info, "Default Info".to_string()))]
pub struct MainTable {
    #[same_as(satellite::parent_id)]
    id: i32,
    #[mandatory(satellite)]
    satellite_id: i32,
    name: String,
}

fn main() {}
